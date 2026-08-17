-- Refracted-owned Shell patch.
-- Canonical source for the Lua string embedded in Prism

local function refractedInstallUsersettingsGraphics()
  local mod = package.loaded["shellmodules.usersettings"]
  if type(mod) ~= "table" then
    return false
  end
  if mod._refractedGraphicsExt == true then
    return true
  end

  if type(getLuaOptionInt) ~= "function" or type(setLuaOptionInt) ~= "function" then
    print("[refracted] graphics bridge: get/setLuaOptionInt missing")
    return false
  end

  mod._refractedGraphicsExt = true

  local function optionBinding(settingsKey)
    return {
      function()
        local ok, value = pcall(getLuaOptionInt, settingsKey)
        if ok then
          return value
        end
        return nil
      end,
      function(nextValue)
        pcall(setLuaOptionInt, settingsKey, nextValue)
      end,
    }
  end

  local function clampedIntBinding(settingsKey, maxInclusive)
    return {
      function()
        local ok, value = pcall(getLuaOptionInt, settingsKey)
        if ok then
          return value
        end
        return nil
      end,
      function(nextValue)
        local n = tonumber(nextValue)
        if n == nil then
          return
        end
        n = math.floor(n + 0.5)
        if n < 0 then
          n = 0
        elseif n > maxInclusive then
          n = maxInclusive
        end
        pcall(setLuaOptionInt, settingsKey, n)
      end,
    }
  end

  -- Brightness is a float option (PostProcess.UIBrightnessNorm).
  -- Must use get/setLuaOptionNumber: int path truncates 0.5→0 (only 100% works).
  -- Prism setLuaOptionNumber also coerces a leftover int-typed option slot to float.
  local function brightnessBinding()
    local getFn = (type(getLuaOptionNumber) == "function") and getLuaOptionNumber or getLuaOptionInt
    local setFn = (type(setLuaOptionNumber) == "function") and setLuaOptionNumber or setLuaOptionInt
    return {
      function()
        local ok, value = pcall(getFn, "Brightness")
        if ok then
          return value
        end
        return nil
      end,
      function(nextValue)
        pcall(setFn, "Brightness", nextValue)
      end,
    }
  end

  local graphicsBindings = {
    overallgraphicsquality = "OverallGraphicsQuality",
    texturequality = "TextureQuality",
    shadowquality = "ShadowQuality",
    effectsquality = "EffectsQuality",
    meshquality = "MeshQuality",
    terrainquality = "TerrainQuality",
    antialiasingpost = "AntiAliasingPost",
    ambientocclusion = "AmbientOcclusion",
    vsyncenabled = "VSyncEnabled",
    motionblurenabled = "MotionBlurEnable",
  }

  for jsonKey, settingsKey in pairs(graphicsBindings) do
    mod.userSettings[jsonKey] = optionBinding(settingsKey)
  end
  mod.userSettings.anisotropicfilter = clampedIntBinding("AnisotropicFilter", 2)
  mod.userSettings.brightness = brightnessBinding()

  -- Coerce JSON/WebKit values into types RtsProfileSettings expects.
  local function toProfileBool(v)
    if type(v) == "boolean" then return v end
    if type(v) == "number" then return v ~= 0 end
    if type(v) == "string" then
      local s = string.lower(v)
      if s == "true" or s == "1" then return true end
      if s == "false" or s == "0" then return false end
    end
    return v
  end

  -- Rts.AllowCameraRotation is a UI console var, not SettingsManager / RtsProfileSettings.
  -- getUserOptions("AllowCameraRotation") asserts (Setting not defined); pcall cannot catch it.
  -- Persist via Prism get/setAllowCameraRotation → PrismOptions.cfg.
  local function cameraRotationBinding()
    return {
      function()
        if type(getAllowCameraRotation) == "function" then
          local ok, value = pcall(getAllowCameraRotation)
          if ok and value ~= nil then
            return value
          end
        end
        return 0
      end,
      function(nextValue)
        local on = toProfileBool(nextValue)
        if type(setAllowCameraRotation) == "function" then
          pcall(setAllowCameraRotation, on and 1 or 0)
        end
        if type(executeConsoleCommand) == "function" then
          pcall(executeConsoleCommand,
            "Rts.AllowCameraRotation " .. (on and "true" or "false"))
        end
      end,
    }
  end

  mod.userSettings.allowcamerarotation = cameraRotationBinding()

  -- Push saved profile camera flags to live console vars (launch + after applyProfileSettings).
  function mod.syncProfileCameraConsole()
    if type(executeConsoleCommand) ~= "function" then
      return
    end
    local mmbDrag = false
    local dragBinding = mod.userSettings.middlemousecameradrag
    if type(dragBinding) == "table" and type(dragBinding[1]) == "function" then
      local ok, value = pcall(dragBinding[1])
      if ok then
        mmbDrag = toProfileBool(value)
      end
    end
    local rotOn = false
    if not mmbDrag then
      local rotBinding = mod.userSettings.allowcamerarotation
      if type(rotBinding) == "table" and type(rotBinding[1]) == "function" then
        local ok, value = pcall(rotBinding[1])
        if ok then
          rotOn = toProfileBool(value)
        end
      end
    end
    pcall(executeConsoleCommand,
      "Rts.AllowCameraRotation " .. (rotOn and "true" or "false"))
    local drag = nil
    if type(dragBinding) == "table" and type(dragBinding[1]) == "function" then
      local ok, value = pcall(dragBinding[1])
      if ok then
        drag = value
      end
    end
    if drag ~= nil then
      local on = toProfileBool(drag)
      pcall(executeConsoleCommand,
        "Rts.IssueRightClickOnMouseDown " .. (on and "true" or "false"))
    end
  end

  local graphicsKeys = {
    overallgraphicsquality = true,
    texturequality = true,
    shadowquality = true,
    effectsquality = true,
    meshquality = true,
    terrainquality = true,
    antialiasingpost = true,
    ambientocclusion = true,
    anisotropicfilter = true,
    vsyncenabled = true,
    motionblurenabled = true,
    brightness = true,
  }

  local profileBoolKeys = {
    shellfullscreen = true,
    gamefullscreen = true,
    edgepan = true,
    middlemousecameradrag = true,
    allowcamerarotation = true,
    movemodeattack = true,
    allowdeselect = true,
  }

  local function applyFromPayload(payload)
    payload = payload or {}
    if payload.middlemousecameradrag ~= nil and toProfileBool(payload.middlemousecameradrag) then
      payload.allowcamerarotation = 0
    end
    local touchedProfile = false
    local touchedGraphics = false
    for key, binding in pairs(mod.userSettings) do
      local value = payload[key]
      if value ~= nil then
        if profileBoolKeys[key] then
          value = toProfileBool(value)
        end
        pcall(binding[2], value)
        if graphicsKeys[key] then
          touchedGraphics = true
        else
          touchedProfile = true
        end
      end
    end
    -- Graphics applyTypedSettings(1) can stomp RenderDevice.Fullscreen and
    -- Rts.IssueRightClickOnMouseDown (camera drag button). Always apply profile
    -- AFTER graphics, then force the camera console var from Settings.
    if touchedGraphics and type(applyLuaOptionGraphics) == "function" then
      pcall(applyLuaOptionGraphics)
    elseif touchedGraphics then
      print("[refracted] applyLuaOptionGraphics missing; graphics may need client rebuild")
    end
    if (touchedProfile or touchedGraphics) and type(applyProfileSettings) == "function" then
      pcall(applyProfileSettings)
    end
    mod.syncProfileCameraConsole()
  end

  function mod.executeUserSettingsApplyRequest(payload)
    applyFromPayload(payload or {})
    local response = { status = 0 }
    if payload and payload._response ~= nil then
      postJsonResult(payload._response, response)
    end
  end

  function mod.executeUserSettingsRequest(payload)
    local response = { status = 0 }
    local filled = 0
    for key, binding in pairs(mod.userSettings) do
      local ok, value = pcall(binding[1])
      if ok and value ~= nil then
        response[key] = value
        filled = filled + 1
      end
    end
    print("[refracted] usersettings get filled=" .. tostring(filled))
    if payload and payload._response ~= nil then
      postJsonResult(payload._response, response)
    end
  end

  -- Retail saveProfileSettings → RtsProfileSettings.cfg; native → PrismOptions.cfg.
  function mod.executeUserSettingsSaveRequest(payload)
    if type(saveProfileSettings) == "function" then
      pcall(saveProfileSettings)
    end
    if type(saveRefractedGraphics) == "function" then
      pcall(saveRefractedGraphics)
    else
      print("[refracted] saveRefractedGraphics missing; graphics may need client rebuild")
    end
    local response = { status = 0 }
    if payload and payload._response ~= nil then
      postJsonResult(payload._response, response)
    end
  end

  local rebound = 0
  if type(_ShellServiceRoutes) == "table" then
    for _, route in ipairs(_ShellServiceRoutes) do
      local path = route[1]
      if path == "/usersettings/apply" then
        route[3] = mod.executeUserSettingsApplyRequest
        rebound = rebound + 1
      elseif path == "/usersettings" then
        route[3] = mod.executeUserSettingsRequest
        rebound = rebound + 1
      elseif path == "/usersettings/save" then
        route[3] = mod.executeUserSettingsSaveRequest
        rebound = rebound + 1
      end
    end
  end

  print("[refracted] graphics bridge: usersettings schema patch applied routes=" .. tostring(rebound))
  return true
end

if not refractedInstallUsersettingsGraphics() and rawget(_G, "__refracted_usersettings_watch") ~= true then
  _G.__refracted_usersettings_watch = true
  local loaded = package.loaded
  local mt = getmetatable(loaded)
  if type(mt) ~= "table" then
    mt = {}
    setmetatable(loaded, mt)
  end
  local prevNewIndex = mt.__newindex
  mt.__newindex = function(t, k, v)
    if type(prevNewIndex) == "function" then
      prevNewIndex(t, k, v)
    else
      rawset(t, k, v)
    end
    if k == "shellmodules.usersettings" then
      refractedInstallUsersettingsGraphics()
    end
  end
end
