# Command & Conquer — shell UI (in-game WebKit host)

The shell runs in the game's embedded WebKit: EAWebKit 16.4 (`EAWebKit16.dll` behind Prism's facade
`EAWebkit.dll`), WebKit of late 2015 (flexbox, transforms, transitions, filters, ES6). It draws Classic,
Aurora (Default) and Aurora (Alternate).

There is no 13.2 fallback. If `EAWebKit16.dll` cannot be loaded the facade shows an error and stops the
game (`prism/titles/cnc/eawebkit/README.md`). See "Compatibility" for what the engine allows.

## Stack (locked)

| Lib | Version |
|-----|---------|
| jQuery | 1.9.1 |
| AngularJS | 1.1.5 |
| Bootstrap | 2.4.2 |

## UI themes

Pickable from **OPTIONS → INTERFACE**:

| Id | Name | Root |
|----|------|------|
| `classic` | Classic | Original Generals 2 layout (`view/roots/classic.html` + `view/home.html`) |
| `aurora` | Aurora (Default) | Concept sheet 2G-1: your general over the battlefield, gunmetal panels (`view/roots/aurora.html` + `view/home-aurora.html`) |
| `aurora-alt` | Aurora (Alternate) | Concept sheet 2A-3: same templates; classic backdrop, cyan readouts, wide feature banner |

Persistence (per-machine, per game install):
1. Shell UI theme — `localStorage` key `cnc_shell_ui_theme` in the shell WebKit profile (`CNCO_DL\0\webkit` on retail)
2. Lobby defaults — `localStorage` key `cnc_lobby_defaults` in the same WebKit profile

These are client-side only. Refracted does not write shell prefs to its host filesystem.

Legacy id `cnc-alpha` maps to `classic`.

### Stylesheets

Every `<link rel="stylesheet" data-shell-css="...">` names the theme it belongs to, and
`js/shell-theme.js` keeps only the active theme's links in the document. So the classic stylesheets
(Bootstrap, `css/*.css`) and the Aurora ones never mix, and an Aurora page is styled by exactly one file:

| Theme | Stylesheet |
|-------|------------|
| Classic | `lib/bootstrap/*.css`, `css/*.css`, `css/themes/cnc-alpha.css` |
| Aurora (Default) | `css/themes/aurora.css` |
| Aurora (Alternate) | `css/themes/aurora-alt.css` |

A new page (or a new stylesheet) has to carry the attribute, or it loads under every theme.

### The two Aurora themes

`css/themes/aurora.css` and `css/themes/aurora-alt.css` are **generated**. Edit the source and rebuild:

```text
refracted/tools/cnc-shell-webdev/aurora/src/00-base.css      fonts, reset, frames, buttons, dialogs, tooltips
                                            10-home.css      main menu, news article, section page frame, nav bar
                                            20-lobby.css     lobby, lobby dialogs, server browser, colour picker
                                            30-options.css   game options
                                            40-pages.css     profile, learn, support, credits
                                            50-ingame.css    pause menu, in-game chat
                                            60-motion.css    entrance animations, hover motion
python refracted/tools/cnc-shell-webdev/aurora/build.py
```

This WebKit has no `var()`, so `build.py` substitutes the palette (the `THEMES` table at its top) and
writes one complete stylesheet per theme. Source syntax beyond plain CSS:

| Syntax | Meaning |
|--------|---------|
| `@{accent}`, `@{line/0.26}` | a palette colour, optionally at an alpha |
| `120u` | 120 px on the 1920x1080 reference frame (written out as `7.5rem`) |
| `/*@default*/ ... /*@end*/`, `/*@alt*/ ... /*@end*/` | rules for one theme only |

Both themes share one accent, light blue (`accent` in the palette); ready green, quit red and the
house colours are not themed. Corner ticks are the `tick` token, at 40% in both themes (corner blue in
Default, cyan in Alternate), and only on the main menu cards and the lobby's player cards, spectator box
and lobby options; other panels and modals have the hairline border alone. Template differences between the two (feature banner, player card,
which stats block shows) are `ng-if="shellUiTheme == ..."` in `view/home-aurora.html`.

### Layout scale

The Aurora pages are measured on the concept sheets' 1920x1080 frame and written in `rem`.
`CncShellTheme.fitRoot()` sets the root font size to `16px x min(width / 1920, height / 1080)` on load
and resize, so the layout keeps its proportions at any resolution: wider windows gain space between the
left and right blocks, taller ones gain space above the nav bar. A page drawn in a small view declares its
own frame with `<html data-au-ref="WxH">` (`ingame_chat.html`: `630x245`).

Aurora (Default) shows the preferred general (OPTIONS → GAMEPLAY → Lobby defaults) on the main menu and
bust portraits on lobby cards. The general shown is the one the player last played (Refracted
records it with each game report: `/cnc/match-record` `lastGeneralId`), else the preferred general.
`js/shell-general-art.js` maps a general to its menu art (next section); the lobby busts come from the
game's `UTFWinAssets\images\UserPortraits` through `/cnc/utfwin/...`.

### General art (main menu)

The general stands in a **square box on the nav bar, against the right edge**, as tall as the card
column beside it (753 px on the 1920x1080 frame; it scales with the window like the rest of the page):

| Window | Box |
|--------|-----|
| 1280x720 | 502 x 502 |
| 1920x1080 | 753 x 753 |
| 2560x1440 | 1004 x 1004 |
| 3840x2160 | 1506 x 1506 |

Menu art is one cut-out per general in `images/general-art/`, mapped by `MENU_ART` in
`js/shell-general-art.js` (the files keep the game's art names):

| General | File | General | File |
|---|---|---|---|
| APA Classic | `AG_Basic_01_UP.png` | EU Laser | `EG_Railgun_01_UP.png` |
| APA Atomic | `AG_Atomic_01.png` | EU Rapid Assault | `EGRapidAssault01.png` |
| APA Espionage | `AGEspionage01.png` | EU Recon | `EGRecon01.png` |
| APA Flame | `AG_Tank_01_UP.png` | EU Science | `EG_ExperimentalWeapons_01_UP.png` |
| APA Rocket | `AGRocket01.png` | GLA Classic | `GGBasic01.png` |
| APA Urban | `AGUrbanCombat01.png` | GLA Demolition | `GGDemolition01.png` |
| EU Classic | `EgBasic01.png` | GLA Engineer | `GGEngineer01.png` |
| EU Airforce | `EGAirForce01.png` | GLA Junkyard | `GGSalvage01.png` |
| GLA Toxin | `GGToxin01.png` | GLA Marauder | none yet: game loading screen |

A tutorial general wears its faction's Classic art. Each image is fitted inside the box and anchored
bottom right, so a square fills it exactly.

| | |
|---|---|
| Ideal size | **1536 x 1536**, PNG with alpha (1:1 at 4K); the supplied set is about 1000-1250 px tall |
| Figure | cut out on transparency, touching the bottom edge |
| Keep clear | the record plate covers the lower right of the box |

A general without menu art (Marauder) falls back to the game's 1920x1080 loading-screen art, scaled to
the box's height and slid right by its `edge`.

### War maps (lobby map stage)

The Aurora lobby's map stage is **14:9**:

| Window | Stage |
|--------|-------|
| 1280x720 | 541 x 348 |
| 1920x1080 | 812 x 522 |
| 2560x1440 | 1083 x 696 |
| 3840x2160 | 1624 x 1044 |

(It was 952 x 612 until the player cards were widened to 468 at 1080p; art made for that size still
fits, as the stage keeps its 14:9 ratio.)

A war map fills the whole stage. It is one file per map, named after the map's minimap file and found
by probing (`.jpg` first, then `.png`):

```text
view/image/lobby-warmaps/<minimap name>.jpg      e.g. view/image/lobby-warmaps/DM_Monsoon_3v3_MO.jpg
```

| | |
|---|---|
| Ideal size | **1904 x 1224** (14:9; covers 4K with room), JPG around quality 85 |
| Minimum | 812 x 522 |
| Other ratios | drawn centred with `cover`, so the excess is cropped evenly |
| Colour | natural: the stylesheet drains it (Default) or tints it cyan (Alternate) and lays the grid and vignette on top |
| Keep clear | the map name chip (top left, about 36% x 11%) and the Change map button (top right, about 17% x 11%) |

Minimap names: `Alpha_Tutorial`, `DM_Smalltown_1v1_CR`, `DM_KapuKai_1v1_JKS`, `DM_Oasis_2v2_JT`,
`DM_Monsoon_3v3_MO`, `DM_Overpass_3v3_JKS`, `FirstPlayable_MPHorde_Final`.

**Start pips** keep the `u`/`v` they have on the square lobby minimap (`starts` in `MAPS`,
`js/shell-lobby.js`) and are placed through `warFit`: the rectangle the square minimap occupies inside
the war map, in fractions of the war map (`u0`,`v0` top left, `u1`,`v1` bottom right; values outside
0..1 mean the square runs past the picture).

| How the war map was made | `warFit` |
|--------------------------|----------|
| The square minimap scaled to the stage's width and cropped top and bottom (the default, nothing to add) | `{ u0: 0, v0: -0.278, u1: 1, v1: 1.278 }` |
| A wider picture with the square minimap in the middle at full height | `{ u0: 0.179, v0: 0, u1: 0.821, v1: 1 }` |
| The square minimap stretched to 14:9 | `{ u0: 0, v0: 0, u1: 1, v1: 1 }` |
| Anything else | measure where the minimap's four edges fall in the war map |

Add `warFit: { ... }` to the map's entry in `MAPS` when the default does not match. With the default,
a start whose minimap `v` is outside 0.18..0.82 falls off the stage and is pinned to its edge, so maps
with starts near the top or bottom (Monsoon, Oasis, Kapu Kai, Smalltown) want a wider picture and
their own `warFit`.

A map without a war map shows its square lobby minimap (`view/image/lobby-minimaps/`, 522 x 522 on a
1080p window) in the middle of the stage over a blurred copy of itself.

The map picker's preview stays square (420 x 420 at 1080p, the lobby minimap) and the server browser
card stays 2:1 (462 x 231 at 1080p, `view/image/lobby-card-art/`, 1024 x 512 files).

**Find a match** (both Aurora themes) has no map step: `$rootScope.quickMatch()` opens the lobby in its
`matchmake` state, which only searches. It pings every joinable server (`/cnc/server-ping`, 1.5 s cap) and joins
the lowest-ping one that already has players (ties go to the busier lobby), or the lowest-ping idle server when
none has players. The host of an idle server picks the map in the lobby with Change map. Any other arrival at
`matchmake` (leaving a lobby, Back from the server browser, a lost server) returns to the main menu. Classic keeps
its map-pick screen.

**Options** in both Aurora themes, on the main shell and in the pause menu, is a full page: one setting
per row with accent section heads, Gameplay first (lobby defaults at its top), radio choices as a segmented
bar (`opt-radio--on`), checkboxes as switches, sliders as tick rows, and Restore defaults / Cancel / Apply.
The markup is `view/options.html`, shared with Classic; `auroraOptions` in `options.js` switches the few
pieces that differ (section heads, segmented buttons and colour swatches in place of three dropdowns).

The Aurora server browser lists every server (there is no map pick to filter by).

The menu art ships with the shell (`images/general-art/`). Lobby busts, minimaps, the spectator icon and the
Marauder fallback come from the game's `UTFWinAssets` through `/cnc/utfwin/...` (`CNC_GAME_DIR` or `CNC_UTFWIN_DIR`).

Refracted serves the shell from its build output (`target\<profile>\data\client\cnc`), which the build
copies from `src\client\cnc\data`. After editing the shell, rebuild Refracted or copy the changed files
there too, or the game keeps showing the old shell.

Tooltips: put `data-tip="..."` on a control (`js/shell-lobby-tooltips.js`); a disabled button needs the
attribute on a wrapper, because it gets no mouse events.

Aurora main menu: the Play page carries four cards (Find a match, Browse servers, Support, Profile);
Profile holds the Generals page as a sub-tab, and Support holds Learn (Play tutorial, FAQ, Guides,
Troubleshoot; a request for the Learn tab opens Support). The bottom bar has Play, then Options, Support us,
Credits and Quit as ordinary buttons, with the online counts at its right end. The `rfr` / `prism` / `cnc`
versions sit under the logo at half strength and open the About note.

Aurora pages: `view/home-aurora.html` (main menu, news article, section pages), `view/profile/*-aurora.html`,
`view/lobby/lobby-aurora.html` with `view/lobby/slot-aurora.html` (one player card, used by both teams),
`view/lobby/server-browser-aurora.html`. Learn, Support, Credits and Options reuse the classic templates.

Profile pages read `GET /cnc/profile` (Refracted, Nexus mysql): wins, losses, streaks, each reported match
(`persona_match_history`, written from game reports) and the wins leaderboard. Per-faction statistics,
levels, ratings and achievements are not recorded, so the pages say so instead of showing numbers.
The faction rank bars on the record plate (`playerFactions` in `app.js`) are still placeholders.

Lobby slot menu (Aurora): as wide as the player card; faction buttons, general tiles with the 64x64
portrait (`view/image/generals/`), colour swatches. Chat polls merge into the lines already shown, so a
poll no longer re-renders (and re-animates) the log.

Player name colours (Nexus name tags, `users.name_tag`): developer purple, playtester yellow `#ffcf47`,
everyone else plain grey `#b0b0b0` (`name` token in Aurora, the same grey in Classic), on the lobby name
cards, the spectator card and in lobby chat. Never the house colour: that is the colour ribbon's job.
In-game chat is friend or foe as the reader sees it: you and your allies blue, enemies red, team channel
green, spectators grey; a developer or playtester tag wins over that.

Esc in the lobby (`cnc:escape` in `js/shell-lobby.js`): while a held step is showing (finding a server,
joining a lobby, starting the match: `_findingMatch`, `_joinStatus`, `_starting`) Esc is ignored, since
backing out half-way strands the seat or the match on the server. On the error dialog such a step leaves
(`_startError`, `_matchError`, `_lobbyFullError`) Esc is the OK button. Quitting (`quittingOpen`) already
ignores Esc in the shell and in the pause menu. Any new held dialog must be added to that check.

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
HTML; `view/newsbar.html` (Classic) and `view/home-aurora.html` (Aurora) render them with text bindings only.

Aurora layout: in the panel (Default) and the strip (Alternate) the date is a small line above the title,
with a `Pinned` tag in front of it for pinned articles; the feed sends a full date ("Sat Oct 03 2026"), which
does not fit a side column. An article opens as a dialog over the main menu (`.au-article-veil` darkens the
menu, `.au-article` is the 1000u box centred on it): the box is as tall as the article needs and its body
scrolls past 720u. Close, Escape or a click on the veil shuts it.

Sign-in screen: the dots after "Communicating with Refracted" are CSS (`.cc-pre-landing__dots`, three `<i>`,
"... .. . .. ..." in five 0.34 s frames) and run until the screen goes. Do not stop them when the login
answers: the screen stays up for `MIN_PRE_LANDING_MS` after that.

## Test browser

Aurora: any current Chromium or WebKit browser against Refracted, `http://127.0.0.1/cncg2/shell/index.html`
(`?skipLogin=1` skips the sign-in screen). Check the result in-game as well: the engine is WebKit of 2015.

Classic uses the same browser. Chrome 15 (`refracted/ref/cnc support/chrome15/`,
`tools\cnc-shell-webdev\launch-chrome15.ps1`) only matters for comparing against the retail 2013 shell.

## Compatibility

Every theme runs on EAWebKit 16.4. Classic's templates and `css/*.css` were written for the 2013 engine
and are left as they are; new work does not need to follow those rules.

| Available | Not available |
|-----------|---------------|
| flexbox, `calc()`, unprefixed `transform` / `transition` / `animation` | CSS custom properties (`var()`) |
| `filter` (also `-webkit-filter`), `-webkit-mask-image`, `clip-path`, `object-fit` | `backdrop-filter`, `mix-blend-mode` |
| multiple backgrounds, gradients, `background-blend-mode`, `@supports` | CSS grid (only the old `-webkit-grid` draft), flex `gap` |
| `rem`, `vw` / `vh`, SVG, canvas 2D | `min()` / `max()` / `clamp()`, `aspect-ratio`, `position: sticky` |
| ES6 in JS (classes, arrows, `let`, template literals, Promise, Map/Set) | `fetch`, Proxy, WebGL, woff2 fonts |

Existing shell JS is ES5 (Angular 1.1.5 era); ES6 is allowed in new code, but keep `fetch` out.

Layout quirks of this WebKit, found in-game (current browsers hide all of them):

- A `<button>` or `<fieldset>` cannot be a flex container: `display: flex` on it is ignored. Use a `div`,
  or place the content with padding. Both are fine as flex *items*.
- `height: 100%` does not resolve inside a flex item that is only stretched. Give the parent a real height
  (`height`, or absolute `top` + `bottom`), or pin the child with absolute positioning. `flex: 1` inside a
  nested column flexbox does work.
- Text smaller than the engine's `mSmoothFontSize` is drawn without anti-aliasing. Prism's EAWebKit facade
  sets it to 0 (smooth at every size); see `prism/titles/cnc/eawebkit/README.md`.
- An `<img>` or `new Image()` pointed at a missing file is not guaranteed to raise `error`: do not chain a
  fallback on it. Load the fallback and the candidate side by side and let the candidate win on `load`.

## Shoutout

Thanks to DerPlayer for kicking off this framework.
