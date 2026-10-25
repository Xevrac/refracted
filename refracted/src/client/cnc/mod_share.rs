//! Coordination for player-to-player mod downloads (refracted.toolset docs/specs/content-lobby-mods.md).
//!
//! The backend never carries mod bytes. It keeps, per lobby, who shares which mods at which
//! address (checked reachable once with a PING), issues short-lived tokens the sharer checks
//! offline, matches downloaders that need a reverse connection with a sharer, and collects
//! progress for the lobby. Prism-facing answers are plain text lines; the shell reads JSON.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;
use rand::Rng;
use ring::hmac;
use serde_json::{json, Value};

use super::{dedicated_pool, game_state};
use crate::http::handlers::handlers_module::{current_http_peer, HttpResponse};

const TOKEN_SECONDS: i64 = 600;
const REVERSE_SECONDS: u64 = 120;
const PROGRESS_SECONDS: u64 = 600;

#[derive(Clone)]
struct Sharer {
    pid: i64,
    ip: String,
    port: u16,
    secret: Vec<u8>,
    mods: Vec<String>,
    reachable: bool,
    method: String,
    error: String,
}

#[derive(Clone)]
struct ReverseRequest {
    pid: i64,
    ip: String,
    port: u16,
    nonce: String,
    mods: Vec<String>,
    at: u64,
    claimed: bool,
}

#[derive(Clone)]
struct Progress {
    mod_id: String,
    state: String,
    done: u64,
    total: u64,
    error: String,
    at: u64,
}

#[derive(Default)]
struct Lobby {
    sharers: Vec<Sharer>,
    reverse: Vec<ReverseRequest>,
    progress: HashMap<i64, Progress>,
    content: HashMap<i64, String>,
}

fn lobbies() -> &'static Mutex<HashMap<i64, Lobby>> {
    static M: OnceLock<Mutex<HashMap<i64, Lobby>>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(HashMap::new()))
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < b.len() => {
                match std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                    Some(v) => {
                        out.push(v);
                        i += 2;
                    }
                    None => out.push(b'%'),
                }
            }
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn param(query: Option<&str>, key: &str) -> String {
    query
        .unwrap_or("")
        .split('&')
        .find_map(|p| p.split_once('=').filter(|(k, _)| *k == key).map(|(_, v)| decode(v)))
        .unwrap_or_default()
}

fn param_i64(query: Option<&str>, key: &str) -> i64 {
    param(query, key).trim().parse().unwrap_or(0)
}

fn is_mod_id(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

fn mod_list(s: &str) -> Vec<String> {
    s.split(',').filter(|id| is_mod_id(id)).take(64).map(str::to_string).collect()
}

fn text(status: u16, body: String) -> HttpResponse {
    HttpResponse::new(status, "text/plain", body.into_bytes())
}

fn in_lobby(gid: i64, pid: i64) -> bool {
    pid > 0 && game_state::players_for_gid(gid).iter().any(|p| p.persona_id == pid && !p.is_ai)
}

fn caller_ip() -> Option<String> {
    current_http_peer().map(|a| a.ip().to_canonical().to_string())
}

fn hex_secret(s: &str) -> Option<Vec<u8>> {
    if s.len() != 64 {
        return None;
    }
    hex::decode(s).ok()
}

/// Same token as prism.content `MakeShareToken`: "<gid>.<pid>.<expiry>.<first 32 bytes of
/// HMAC-SHA-512(secret, "rmod/1|gid|pid|expiry"), hex>".
fn make_token(secret: &[u8], gid: i64, pid: i64, expiry: i64) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA512, secret);
    let tag = hmac::sign(&key, format!("rmod/1|{gid}|{pid}|{expiry}").as_bytes());
    format!("{gid}.{pid}.{expiry}.{}", hex::encode(&tag.as_ref()[..32]))
}

/// RMOD/1 PING: the address really reaches that player's game.
fn ping(ip: &str, port: u16) -> bool {
    let Ok(addr) = format!("{}:{}", if ip.contains(':') { format!("[{ip}]") } else { ip.to_string() }, port).parse::<SocketAddr>()
    else {
        return false;
    };
    let Ok(mut s) = TcpStream::connect_timeout(&addr, Duration::from_secs(3)) else {
        return false;
    };
    let _ = s.set_read_timeout(Some(Duration::from_secs(3)));
    let _ = s.set_write_timeout(Some(Duration::from_secs(3)));
    let nonce: String = (0..16).map(|_| format!("{:02x}", rand::thread_rng().gen::<u8>())).collect();
    if s.write_all(format!("RMOD/1 PING {nonce}\n").as_bytes()).is_err() {
        return false;
    }
    // At most one short line back; the read timeout bounds a peer that never sends one.
    let mut line = String::new();
    let mut reader = BufReader::new(s.take(256));
    reader.read_line(&mut line).is_ok() && line.trim_end() == format!("PONG {nonce}")
}

/// `POST /cnc/mod-share/advertise?gid&pid&port&secret&mods&method&error` — a lobby member shares.
pub fn handle_advertise(query: Option<&str>) -> HttpResponse {
    let gid = param_i64(query, "gid");
    let pid = param_i64(query, "pid");
    if !in_lobby(gid, pid) {
        return text(403, "err not in this lobby\n".into());
    }
    let (Some(ip), Some(secret)) = (caller_ip(), hex_secret(&param(query, "secret"))) else {
        return text(400, "err bad request\n".into());
    };
    let port: u16 = param(query, "port").parse().unwrap_or(0);
    let mods = mod_list(&param(query, "mods"));
    if port == 0 || mods.is_empty() {
        return text(400, "err bad request\n".into());
    }
    let reachable = ping(&ip, port);
    let method: String = param(query, "method").chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').take(16).collect();
    let error: String = param(query, "error").chars().filter(|c| !c.is_control()).take(200).collect();
    let mut m = lobbies().lock();
    let lobby = m.entry(gid).or_default();
    lobby.sharers.retain(|s| s.pid != pid);
    lobby.sharers.push(Sharer { pid, ip: ip.clone(), port, secret, mods, reachable, method, error });
    crate::console_println!("[Mod share] gid={gid} pid={pid} shares at {ip}:{port} reachable={reachable}");
    text(200, format!("ok {} {ip} {port}\n", if reachable { "reachable" } else { "unreachable" }))
}

/// `POST /cnc/mod-share/withdraw?gid&pid`
pub fn handle_withdraw(query: Option<&str>) -> HttpResponse {
    let gid = param_i64(query, "gid");
    let pid = param_i64(query, "pid");
    if let Some(l) = lobbies().lock().get_mut(&gid) {
        l.sharers.retain(|s| s.pid != pid);
    }
    text(200, "ok\n".into())
}

/// `GET /cnc/mod-share/plan?gid&pid&mod` — reachable sharers of a mod, with tokens, and whether a
/// reverse connection could help (someone shares it but can't be reached).
pub fn handle_plan(query: Option<&str>) -> HttpResponse {
    let gid = param_i64(query, "gid");
    let pid = param_i64(query, "pid");
    let id = param(query, "mod");
    if !in_lobby(gid, pid) || !is_mod_id(&id) {
        return text(403, "err not allowed\n".into());
    }
    let expiry = now() as i64 + TOKEN_SECONDS;
    let m = lobbies().lock();
    let mut out = String::new();
    let mut any = false;
    if let Some(l) = m.get(&gid) {
        for s in l.sharers.iter().filter(|s| s.pid != pid && s.mods.contains(&id)) {
            any = true;
            if s.reachable {
                out += &format!("source {} {} {}\n", s.ip, s.port, make_token(&s.secret, gid, pid, expiry));
            }
        }
    }
    out += &format!("reverse {}\n", if any { 1 } else { 0 });
    text(200, out)
}

/// `POST /cnc/mod-share/reverse-request?gid&pid&port&nonce&mods` — a downloader with an open port
/// asks a sharer to connect to it.
pub fn handle_reverse_request(query: Option<&str>) -> HttpResponse {
    let gid = param_i64(query, "gid");
    let pid = param_i64(query, "pid");
    if !in_lobby(gid, pid) {
        return text(403, "err not in this lobby\n".into());
    }
    let Some(ip) = caller_ip() else {
        return text(400, "err bad request\n".into());
    };
    let port: u16 = param(query, "port").parse().unwrap_or(0);
    let nonce = param(query, "nonce");
    let mods = mod_list(&param(query, "mods"));
    if port == 0 || nonce.len() != 32 || !nonce.bytes().all(|c| c.is_ascii_hexdigit()) || mods.is_empty() {
        return text(400, "err bad request\n".into());
    }
    if !ping(&ip, port) {
        return text(200, "ok unreachable\n".into());
    }
    let mut m = lobbies().lock();
    let lobby = m.entry(gid).or_default();
    lobby.reverse.retain(|r| r.pid != pid && now().saturating_sub(r.at) < REVERSE_SECONDS);
    lobby.reverse.push(ReverseRequest { pid, ip, port, nonce, mods, at: now(), claimed: false });
    text(200, "ok reachable\n".into())
}

/// `GET /cnc/mod-share/reverse?gid&pid` — a sharer collects downloaders to connect to.
pub fn handle_reverse(query: Option<&str>) -> HttpResponse {
    let gid = param_i64(query, "gid");
    let pid = param_i64(query, "pid");
    let mut out = String::new();
    let mut m = lobbies().lock();
    if let Some(l) = m.get_mut(&gid) {
        let Some(sharer) = l.sharers.iter().find(|s| s.pid == pid).cloned() else {
            return text(200, String::new());
        };
        let t = now();
        l.reverse.retain(|r| t.saturating_sub(r.at) < REVERSE_SECONDS);
        for r in l.reverse.iter_mut().filter(|r| !r.claimed && r.pid != pid) {
            if let Some(id) = r.mods.iter().find(|id| sharer.mods.contains(id)) {
                r.claimed = true;
                out += &format!("push {} {} {} {}\n", r.ip, r.port, r.nonce, id);
            }
        }
    }
    text(200, out)
}

/// `POST /cnc/mod-share/progress?gid&pid&mod&state&done&total&error`
pub fn handle_progress(query: Option<&str>) -> HttpResponse {
    let gid = param_i64(query, "gid");
    let pid = param_i64(query, "pid");
    if !in_lobby(gid, pid) {
        return text(403, "err not in this lobby\n".into());
    }
    let mod_id = param(query, "mod");
    let state = param(query, "state");
    let states = ["connecting", "downloading", "verifying", "installed", "queued", "restart", "failed"];
    if !(is_mod_id(&mod_id) || mod_id == "-") || !states.contains(&state.as_str()) {
        return text(400, "err bad request\n".into());
    }
    let p = Progress {
        mod_id,
        state,
        done: param(query, "done").parse().unwrap_or(0),
        total: param(query, "total").parse().unwrap_or(0),
        error: param(query, "error").chars().filter(|c| !c.is_control()).take(300).collect(),
        at: now(),
    };
    lobbies().lock().entry(gid).or_default().progress.insert(pid, p);
    text(200, "ok\n".into())
}

/// `POST /cnc/mod-share/content?gid&pid&key` — the shell reports the content key its game mounted.
pub fn handle_content(query: Option<&str>) -> HttpResponse {
    let gid = param_i64(query, "gid");
    let pid = param_i64(query, "pid");
    if !in_lobby(gid, pid) {
        return text(403, "err not in this lobby\n".into());
    }
    let Some(key) = dedicated_pool::sanitize_content_key(&param(query, "key")) else {
        return text(400, "err bad key\n".into());
    };
    lobbies().lock().entry(gid).or_default().content.insert(pid, key);
    text(200, "ok\n".into())
}

/// `GET /cnc/mod-share/status?gid` — for the shell: who shares, and each player's download and content key.
pub fn handle_status(query: Option<&str>) -> HttpResponse {
    let gid = param_i64(query, "gid");
    let server_content = game_state::dedicated_session_for_gid(gid)
        .and_then(dedicated_pool::get_entry)
        .and_then(|e| e.content);
    let t = now();
    let players: Vec<i64> = game_state::players_for_gid(gid).iter().filter(|p| !p.is_ai).map(|p| p.persona_id).collect();
    let mut m = lobbies().lock();
    let lobby = m.entry(gid).or_default();
    lobby.sharers.retain(|s| players.contains(&s.pid));
    lobby.progress.retain(|pid, p| players.contains(pid) && t.saturating_sub(p.at) < PROGRESS_SECONDS);
    lobby.content.retain(|pid, _| players.contains(pid));
    let sharers: Vec<Value> = lobby
        .sharers
        .iter()
        .map(|s| json!({ "pid": s.pid, "reachable": s.reachable, "method": s.method, "error": s.error, "mods": s.mods }))
        .collect();
    let progress: Vec<Value> = lobby
        .progress
        .iter()
        .map(|(pid, p)| json!({ "pid": pid, "mod": p.mod_id, "state": p.state, "done": p.done, "total": p.total, "error": p.error }))
        .collect();
    let content: Value = lobby.content.iter().map(|(pid, k)| (pid.to_string(), Value::String(k.clone()))).collect::<serde_json::Map<_, _>>().into();
    let body = json!({ "ok": true, "gid": gid, "serverContent": server_content, "sharers": sharers, "progress": progress, "content": content });
    HttpResponse::new(200, "application/json", body.to_string().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_matches_the_prism_format() {
        let secret = vec![7u8; 32];
        let t = make_token(&secret, 12, 345, 1_800_000_000);
        let parts: Vec<&str> = t.split('.').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(&parts[..3], &["12", "345", "1800000000"]);
        assert_eq!(parts[3].len(), 64);
        assert_ne!(make_token(&secret, 12, 346, 1_800_000_000), t);
    }

    #[test]
    fn parses_queries() {
        let q = Some("gid=5&pid=42&mods=bad,eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee&error=no%20port");
        assert_eq!(param_i64(q, "gid"), 5);
        assert_eq!(mod_list(&param(q, "mods")).len(), 1);
        assert_eq!(param(q, "error"), "no port");
        assert_eq!(param(q, "missing"), "");
    }
}
