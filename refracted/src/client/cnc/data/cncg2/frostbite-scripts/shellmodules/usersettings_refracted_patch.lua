-- Refracted-owned Shell patch (not a retail Script copy).
-- Canonical source for the Lua string embedded in Prism
-- (CncUserSettingsGraphicsBridge.cpp). Keep both in sync.
-- Extends the in-memory retail table with graphics Settings keys and rebinds
-- /usersettings get+apply handlers. Do not replace usersettings.lua from initfs.
--
-- Graphics keys use getLuaOptionInt / setLuaOptionInt (Prism natives →
-- LuaOptionSetManager). Retail setUserOptions only covers RtsProfileSettings.*.

local mod = package.loaded["shellmodules.usersettings"]
if type(mod) ~= "table" or mod._refractedGraphicsExt == true then
  return
end
mod._refractedGraphicsExt = true

if type(getLuaOptionInt) ~= "function" or type(setLuaOptionInt) ~= "function" then
  print("[refracted] graphics bridge: get/setLuaOptionInt missing")
  return
end

local function optionBinding(settingsKey)
  return {
    function()
      return getLuaOptionInt(settingsKey)
    end,
    function(nextValue)
      setLuaOptionInt(settingsKey, nextValue)
    end,
  }
end

-- JSON field -> LuaOptionSetManager option name.
local graphicsBindings = {
  overallgraphicsquality = "OverallGraphicsQuality",
  texturequality = "TextureQuality",
  shadowquality = "ShadowQuality",
  effectsquality = "EffectsQuality",
  meshquality = "MeshQuality",
  terrainquality = "TerrainQuality",
  antialiasingpost = "AntiAliasingPost",
  ambientocclusion = "AmbientOcclusion",
  anisotropicfilter = "AnisotropicFilter",
  vsyncenabled = "VSyncEnabled",
  motionblurenabled = "MotionBlurEnable",
  brightness = "Brightness",
}

for jsonKey, settingsKey in pairs(graphicsBindings) do
  mod.userSettings[jsonKey] = optionBinding(settingsKey)
end

local function applyFromPayload(payload)
  for key, binding in pairs(mod.userSettings) do
    if payload[key] ~= nil then
      binding[2](payload[key])
    end
  end
  -- Retail display/controls apply; Prism hooks this to also run LuaOptionSetManager::applySettings.
  applyProfileSettings()
end

local function readAllSettings()
  local snapshot = {}
  for key, binding in pairs(mod.userSettings) do
    snapshot[key] = binding[1]()
  end
  return snapshot
end

function mod.executeUserSettingsApplyRequest(payload)
  applyFromPayload(payload or {})
  local response = { status = 0 }
  postJsonResult(payload._response, response)
end

function mod.executeUserSettingsRequest(payload)
  local response = readAllSettings()
  response.status = 0
  postJsonResult(payload._response, response)
end

-- Route table keeps a function reference from first AddServiceRoutes; swap in place.
if type(_ShellServiceRoutes) == "table" then
  for _, route in ipairs(_ShellServiceRoutes) do
    local path = route[1]
    if path == "/usersettings/apply" then
      route[3] = mod.executeUserSettingsApplyRequest
    elseif path == "/usersettings" then
      route[3] = mod.executeUserSettingsRequest
    end
  end
end
