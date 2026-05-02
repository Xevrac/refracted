function getOptions() {
    var request = {
        url: "/options/graphics/get"
    };
    shellaccesslayer.execute(request);
}

function FrontEndFullscreenTrue() {
    var request = {
        _resource: "/usersettings/apply",
        shellfullscreen: true
    };
    shellaccesslayer.execute(request);
}

function FrontEndFullscreenFalse() {
    var request = {
        _resource: "/usersettings/apply",
        shellfullscreen: false
    };
    shellaccesslayer.execute(request);
}

function GameFullscreen() {
    var request = {
        _resource: "/usersettings/apply",
        gamefullscreen: true
    };
    shellaccesslayer.execute(request);
}

function SetFullscreenDimensions() {
    var request = {
        _resource: "/usersettings/apply",
        fullscreenwidth: 1920,
        fullscreenheight: 1080
    };
    shellaccesslayer.execute(request);
}

function SetWindowedDimensions() {
    var request = {
        _resource: "/usersettings/apply",
        windowedwidth: 1600,
        windowedheight: 900
    };
    shellaccesslayer.execute(request);
}

function updateVolume(value) {
    var maxDisplayedValue = 10;
    var displayedValue = (value / 100) * maxDisplayedValue;

    var fullRangeValue = (displayedValue / maxDisplayedValue) * 100;

    document.getElementById('volumeValue').textContent = fullRangeValue.toFixed(0) + '%';
    MasterVolume(displayedValue);
}

function MasterVolume(displayedValue) {
    var maxSliderValue = 100;
    var actualValue = (displayedValue / maxSliderValue) * 100;

    var request = {
        _resource: "/usersettings/apply",
        mastervolume: actualValue
    };

    shellaccesslayer.execute(request);
}

function applyAudio() {
    var request = {
        _resource: "/usersettings/applyAudio"
    };
    shellaccesslayer.execute(request);
}

function save() {
    var request = {
        _resource: "/usersettings/save"
    };
    shellaccesslayer.execute(request);
}

function discard() {
    var request = {
        _resource: "/usersettings/discard"
    };
    shellaccesslayer.execute(request);
}

function executeExpression() {
    var expression = document.getElementById('expressionInput').value;

    try {
        var result = eval(expression);
        document.getElementById('resultBox').innerText = 'Result: ' + result;
    } catch (error) {
        document.getElementById('resultBox').innerText = 'Error: ' + error.message;
    }
}

function ShowObjects() {
    var command = document.getElementById('expressionInput').value;
    var resultBox = document.getElementById('resultBox');

    try {
        var objs = Object.getOwnPropertyNames(eval(command));
        resultBox.innerText = 'Objects: ' + objs.join(', ');
    } catch (error) {
        resultBox.innerText = 'Error: ' + error.message;
    }
}

function refreshWindow() {
    location.reload();
}

function quitClient() {
    gameclient.execute('RTSClient.Quit');
}

var drawScreenInfoEnabled = false;

function toggleDrawScreenInfo() {
    drawScreenInfoEnabled = !drawScreenInfoEnabled;
    drawScreenInfo(drawScreenInfoEnabled);
}

function drawScreenInfo(enable) {
    if (enable) {
        gameclient.execute('Render.DrawInfo true');
        gameclient.execute('Render.DrawFpsHistogram true');
        gameclient.execute('Render.DrawScreenInfo true');
    } else {
        gameclient.execute('Render.DrawInfo false');
        gameclient.execute('Render.DrawFpsHistogram false');
        gameclient.execute('Render.DrawScreenInfo false');
    }
}

var fpsIncreaseEnabled = false;

function toggleFpsIncrease() {
    fpsIncreaseEnabled = !fpsIncreaseEnabled;
    fpsIncrease(fpsIncreaseEnabled);
}

function fpsIncrease(enable) {
    if (enable) {
        gameclient.execute('GameTime.MaxSimFps 60');
        gameclient.execute('GameTime.MaxVariableFps 60');
        gameclient.execute('GameTime.MaxInactiveVariableFps 60');
    } else {
        gameclient.execute('GameTime.MaxSimFps 30');
        gameclient.execute('GameTime.MaxVariableFps 30');
        gameclient.execute('GameTime.MaxInactiveVariableFps 30');
    }
}

function createGame() {
    var configFile = 'C:\\CNCO\\level.cfg';
    var levelName = 'Levels/SP/Alpha_Tutorial/Alpha_Tutorial';
    var playerId = 1;

    var command = 'RtsClient.createGame ' + configFile + ' ' + levelName + ' ' + playerId;
    console.log('Executing command:', command);

    try {
        gameclient.execute(command);
    } catch (error) {
        console.error('Error executing command:', error);
    }
}

function createBlazeGame() {
    shellaccesslayer.execute({
        _response: ShellResult,
        url: "/blaze/createGame?gameName=Player1&players=4"
    });
}

function blazeAuthenticate() {
    var request = {
        _resource: "/blaze/authenticate",
        email: "test@test.com",
        password: "test"
    };

    shellaccesslayer.execute({
        _response: ShellResult,
        url: "/blaze/authenticate?email=" + request.email + "&password=" + request.password
    });
}

function tokenAuthenticate() {
    shellaccesslayer.execute({
        _response: ShellResult,
        url: "/blaze/tokenauthenticate"
    });
}

function ShellResult(res) {
    if (window.CncBlazeState && typeof CncBlazeState.onShellResult === "function") {
        CncBlazeState.onShellResult(res);
    }
    var msg;
    try {
        msg = "RESPONSE: " + JSON.stringify(res, null, 2);
    } catch (e) {
        msg = "RESPONSE: " + String(res);
    }
    var el = document.getElementById("debugOutput") || document.getElementById("cnc-probe-mono-out");
    if (el) { el.innerText = msg; }
    if (typeof CncProbe !== "undefined" && CncProbe && CncProbe.log) { CncProbe.log(msg); }
    if (res && res.success) {
        console.log("Authentication successful!");
    } else {
        console.error("Authentication failed. Error: ", res && res.error);
    }
}

function blazeJoinGame() {
    var request = {
        url: "/blaze/joinGame?gameID=1"
    };
    shellaccesslayer.execute(request);
}

function blazeJoinGame1() {
    var hosting = "localhost";
    var playerId = 1;

    var command = 'RtsClient.joinGame ' + playerId + ' ' + hosting + ' ';
    console.log('Executing command:', command);

    try {
        gameclient.execute(command);
    } catch (error) {
        console.error('Error executing command:', error);
    }
}

function getConfig() {
    var request = {
        url: "/config/"
    };
    shellaccesslayer.execute(request);
}

function quitSession() {
    var request = {
        _resource: "/session/quit"
    };
    shellaccesslayer.execute(request);
}

function surrenderSession() {
    var request = {
        _resource: "/session/surrender"
    };
    shellaccesslayer.execute(request);
}
