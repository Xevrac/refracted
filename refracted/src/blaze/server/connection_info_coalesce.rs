fn emit_compact(upsert_key: String, line: &str, count: u32) {
    let ansi = if count <= 1 {
        line.to_string()
    } else {
        format!("{} \x1b[38;2;140;140;140mx{}\x1b[0m", line, count)
    };
    crate::core::console::push_grpc_compact_upsert(upsert_key, &ansi);
}

/// Coalesces repeated identical Blaze traffic [Client→Blaze] / [Server→Blaze] / [Blaze→Client] / [Blaze→Server] lines.
/// Keys must omit volatile fields (e.g. MsgNum) so consecutive pings count as one signature.
pub struct CoalescedBlazeInfo {
    scope: String,
    blaze_session_id: Option<u64>,
    key: Option<String>,
    count: u32,
    line: String,
    seq: u64,
}

impl CoalescedBlazeInfo {
    pub fn new() -> Self {
        Self::new_scoped("GLOBAL")
    }

    pub fn new_scoped(scope: &str) -> Self {
        Self {
            scope: scope.to_string(),
            blaze_session_id: None,
            key: None,
            count: 0,
            line: String::new(),
            seq: 0,
        }
    }

    pub fn set_blaze_session_id(&mut self, blaze_session_id: Option<u64>) {
        self.blaze_session_id = blaze_session_id;
    }

    fn normalize_line(&self, line: String) -> String {
        self.blaze_session_id
            .map(|sid| crate::client::cnc::dedicated_pool::normalize_blaze_wire_log_line(sid, &line))
            .unwrap_or(line)
    }

    pub fn log(&mut self, key: &str, line: String) {
        let line = self.normalize_line(line);
        // Only coalesce known heartbeat-style rows. Other Blaze traffic should log as distinct lines.
        let coalesceable = key.contains("|KA") || key.contains("|IDLEHB");
        if !coalesceable {
            self.flush();
            self.seq = self.seq.saturating_add(1);
            emit_compact(format!("{}|{}|{}", self.scope, key, self.seq), &line, 1);
            return;
        }

        if self.key.as_deref() == Some(key) {
            self.count = self.count.saturating_add(1);
            self.line = line;
            emit_compact(format!("{}|{}", self.scope, key), &self.line, self.count);
        } else {
            self.flush();
            self.key = Some(key.to_string());
            self.count = 1;
            self.line = line;
            emit_compact(format!("{}|{}", self.scope, key), &self.line, self.count);
        }
    }

    pub fn flush(&mut self) {
        if self.count == 0 {
            return;
        }
        self.key = None;
        self.count = 0;
        self.line.clear();
    }
}

impl Drop for CoalescedBlazeInfo {
    fn drop(&mut self) {
        self.flush();
    }
}

pub struct PingBurstCoalescer {
    scope: String,
    blaze_session_id: Option<u64>,
    req_count: u32,
    req_line: String,
    rep_count: u32,
    rep_line: String,
}

impl PingBurstCoalescer {
    pub fn new() -> Self {
        Self::new_scoped("GLOBAL")
    }

    pub fn new_scoped(scope: &str) -> Self {
        Self {
            scope: scope.to_string(),
            blaze_session_id: None,
            req_count: 0,
            req_line: String::new(),
            rep_count: 0,
            rep_line: String::new(),
        }
    }

    pub fn set_blaze_session_id(&mut self, blaze_session_id: Option<u64>) {
        self.blaze_session_id = blaze_session_id;
    }

    fn normalize_line(&self, line: String) -> String {
        self.blaze_session_id
            .map(|sid| crate::client::cnc::dedicated_pool::normalize_blaze_wire_log_line(sid, &line))
            .unwrap_or(line)
    }

    pub fn log_request(&mut self, line: String) {
        let line = self.normalize_line(line);
        self.req_count = self.req_count.saturating_add(1);
        self.req_line = line;
        emit_compact(
            format!("{}|PING_REQ", self.scope),
            &self.req_line,
            self.req_count,
        );
    }

    pub fn log_reply(&mut self, line: String) {
        let line = self.normalize_line(line);
        self.rep_count = self.rep_count.saturating_add(1);
        self.rep_line = line;
        emit_compact(
            format!("{}|PING_REP", self.scope),
            &self.rep_line,
            self.rep_count,
        );
    }

    pub fn flush(&mut self) {
        // Intentionally no-op: keep cumulative ping counters per Blaze connection.
        // This matches the Web/gRPC compact row behavior where xN keeps increasing.
    }
}

impl Drop for PingBurstCoalescer {
    fn drop(&mut self) {
        self.flush();
    }
}

#[inline]
pub fn key_c2b(component: u16, command: u16, total_size: usize, msg_type: &str) -> String {
    format!("C2B|{component}|{command}|{total_size}|{msg_type}")
}

#[inline]
pub fn key_b2c_reply(component: u16, command: u16, wire_size: usize) -> String {
    format!("B2C|{component}|{command}|R|{wire_size}")
}

#[inline]
pub fn key_b2c_notif(component: u16, command: u16, payload_len: usize) -> String {
    format!("B2C|{component}|{command}|N|{payload_len}")
}

/// Idle 15s heartbeat notification (repeated; stacks when payload size is stable)
#[inline]
pub fn key_b2c_idle_user_session_ex(payload_len: usize) -> String {
    format!("B2C|30722|1|N|{payload_len}|IDLEHB")
}

/// Fire2Frame: keepalive / zero-component
#[inline]
pub fn key_c2b_keepalive(total_size: usize, msg_type: &str) -> String {
    format!("C2B|0|0|{total_size}|{msg_type}")
}

#[inline]
pub fn key_b2c_keepalive_reply() -> String {
    "B2C|0|0|R|16|KA".to_string()
}

#[inline]
pub fn key_b2c_blaze_inject(component: u16, command: u16, msg_num: u32, msg_ty: &str, wire_len: usize) -> String {
    format!("B2C|INJECT|{component}|{command}|{msg_ty}|{msg_num}|{wire_len}")
}

#[inline]
pub fn key_fire_c2b(component: u16, command: u16, total: usize) -> String {
    format!("FR2B|{component}|{command}|{total}")
}

#[inline]
pub fn key_fire_b2c(component: u16, command: u16, response_len: usize) -> String {
    format!("B2FR|{component}|{command}|{response_len}")
}
