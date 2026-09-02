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
