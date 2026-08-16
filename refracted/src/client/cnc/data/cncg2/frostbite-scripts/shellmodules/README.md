# frostbite-scripts / shellmodules

Refracted does **not** ship copies of retail `Scripts/Shell/shellmodules/*.lua`.

| File | Role |
|------|------|
| `usersettings_refracted_patch.lua` | Canonical source for Prism’s post-load graphics Settings patch (embedded in `CncUserSettingsGraphicsBridge.cpp`). Keep in sync when editing either side. |

Graphics quality is applied via Prism natives `getLuaOptionInt` / `setLuaOptionInt` → `LuaOptionSetManager` (not retail `setUserOptions` / `RtsProfileSettings`).
