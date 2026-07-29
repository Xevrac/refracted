# Command & Conquer — shell UI (EAWebKit 13.2)

In-game host is **EAWebKit 13.2.1.0** (WebKit **535.3**). Closest GPL source: [EAWebKit 13.2.0.0](https://gpl.ea.com/packages/EAWebKit_13.3.2.0.0.zip).

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

Persistence:
1. `localStorage` key `cnc_shell_ui_theme` (immediate)
2. Refracted `GET/POST /cnc/shell-theme` → `data/.../cncg2/shell/prefs/ui-theme.json` (survives browser profile wipes)

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
