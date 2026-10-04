# Command & Conquer — shell UI (in-game WebKit host)

The in-game host uses a legacy WebKit-based shell (WebKit ~535-era). Shell scripts target that environment for compatibility (ES5 / WebKit 535-safe CSS).

## Stack (locked)

| Lib | Version |
|-----|---------|
| jQuery | 1.9.1 |
| AngularJS | 1.1.5 |
| Bootstrap | 2.4.2 |

## UI themes

Pickable from **OPTIONS → INTERFACE** on the **main shell only** (not in-game pause):

| Id | Name | Root |
|----|------|------|
| `classic` | Classic | Original Generals 2 layout (`view/roots/classic.html` + `view/home.html`) |
| `aurora` | Aurora | Modern HUD layout (`view/roots/aurora.html` + `view/home-aurora.html`) |

Persistence (per-machine, per game install):
1. Shell UI theme — `localStorage` key `cnc_shell_ui_theme` in the shell WebKit profile (`CNCO_DL\0\webkit` on retail)
2. Lobby defaults — `localStorage` key `cnc_lobby_defaults` in the same WebKit profile

These are client-side only. Refracted does not write shell prefs to its host filesystem.

Legacy id `cnc-alpha` maps to `classic`.
CSS: `css/themes/` (+ `aurora-layout.css` for the Aurora root).

## News (NEWS panel)

Articles come from the Aurora site (`refracted_cnc/frontend`, admin page `/news`), not from this
folder. `js/shell-news.js` polls `GET /cnc/news/{realm}` on Refracted every 5 minutes (refresh button:
30 s cooldown) and keeps the last good feed in `localStorage` (`cnc_news_v1_{realm}`) for offline starts.

| Realm | Page | Picked by |
|-------|------|-----------|
| `prod` | `index.html` / rmtWrapper | default |
| `dev` | `devWrapper.html` | `window.__CNC_PLAYTEST === true` |

Refracted (`client/cnc/news_proxy.rs`) fetches `{news_url}/api/shell/news/{realm}` over HTTPS and
caches it for 30 s; article images go through `GET /cnc/news-image?u=` (site URLs only, cached 10 min).
EAWebKit cannot complete the site's TLS handshake, so the shell never calls the site directly.
Config: `refracted.env` `news_url` / `news_dev_key` (or `CNC_NEWS_URL` / `CNC_NEWS_DEV_KEY`); the key
stays on the server and is only needed on the playtest server.

Articles are typed blocks (`paragraph`, `heading`, `list` with one nested level, `divider`), never
HTML; `view/newsbar.html` renders them with text bindings only.

## Test browser

**Chrome 15.0.875.0** lives under:

```text
refracted/ref/cnc support/chrome15/chrome.exe
```

```powershell
.\refracted\tools\cnc-shell-webdev\launch-chrome15.ps1
```

Opens `http://127.0.0.1/cncg2/shell/index.html` (Refracted on `:80`).

## Compatibility

- ES5 only in shell JS
- No relying on native Promise/fetch without shims
- CSS: WebKit 535-safe (no flex/grid as primary layout, no `var()`)

## Shoutout

Thanks to DerPlayer for kicking off this framework.
