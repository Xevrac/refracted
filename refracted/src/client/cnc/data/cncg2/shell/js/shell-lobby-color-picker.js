/**
 * Lobby house-colour picker (WebKit 535 / ES5).
 */
(function (window, $) {
    'use strict';

    if (!$) {
        return;
    }

    var HUE_BLOCK_DEG = 14;
    var SAT_CHROMA = 0.18;
    var GREY_V_BLOCK = 0.22;
    var SV_W = 172;
    var SV_H = 118;
    var HUE_W = 172;
    var HUE_H = 14;

    var panel = null;
    var svCanvas = null;
    var hueCanvas = null;
    var chipEl = null;
    var hexEl = null;
    var hintEl = null;
    var hostEl = null;
    var onPick = null;
    var taken = [];
    var state = { h: 210, s: 0.72, v: 0.84 };
    var drag = null;
    var hatchAurora = null;
    var hatchClassic = null;

    function isAuroraTheme() {
        return /(?:^|\s)cc-theme--aurora(?:\s|$)/.test(document.body.className || '');
    }

    function parseRgb(hex) {
        var s = String(hex || '').replace(/^\s+|\s+$/g, '').replace(/^#/, '');
        if (s.length === 8) {
            s = s.slice(2);
        }
        if (s.length !== 6 || /[^0-9a-fA-F]/.test(s)) {
            return null;
        }
        return {
            r: parseInt(s.slice(0, 2), 16),
            g: parseInt(s.slice(2, 4), 16),
            b: parseInt(s.slice(4, 6), 16)
        };
    }

    function pad2(n) {
        var s = Math.max(0, Math.min(255, n | 0)).toString(16);
        return s.length < 2 ? '0' + s : s;
    }

    function rgbToHex(r, g, b) {
        return '#' + pad2(r) + pad2(g) + pad2(b);
    }

    function hsvToRgb(h, s, v) {
        var c = v * s;
        var hp = h / 60;
        var x = c * (1 - Math.abs(hp % 2 - 1));
        var m = v - c;
        var r = 0;
        var g = 0;
        var b = 0;
        if (hp < 1) {
            r = c;
            g = x;
        } else if (hp < 2) {
            r = x;
            g = c;
        } else if (hp < 3) {
            g = c;
            b = x;
        } else if (hp < 4) {
            g = x;
            b = c;
        } else if (hp < 5) {
            r = x;
            b = c;
        } else {
            r = c;
            b = x;
        }
        return {
            r: Math.round((r + m) * 255),
            g: Math.round((g + m) * 255),
            b: Math.round((b + m) * 255)
        };
    }

    function rgbToHsv(r, g, b) {
        var rf = r / 255;
        var gf = g / 255;
        var bf = b / 255;
        var max = Math.max(rf, gf, bf);
        var min = Math.min(rf, gf, bf);
        var d = max - min;
        var h = 0;
        if (d !== 0) {
            if (max === rf) {
                h = ((gf - bf) / d) % 6;
            } else if (max === gf) {
                h = (bf - rf) / d + 2;
            } else {
                h = (rf - gf) / d + 4;
            }
            h *= 60;
            if (h < 0) {
                h += 360;
            }
        }
        return {
            h: h,
            s: max === 0 ? 0 : d / max,
            v: max
        };
    }

    function hueDist(a, b) {
        var d = Math.abs(a - b);
        return Math.min(d, 360 - d);
    }

    function hsvOfHex(hex) {
        var rgb = parseRgb(hex);
        if (!rgb) {
            return null;
        }
        return rgbToHsv(rgb.r, rgb.g, rgb.b);
    }

    function tooClose(a, b) {
        var ra = parseRgb(a);
        var rb = parseRgb(b);
        var ha;
        var hb;
        var greyA;
        var greyB;
        if (!ra || !rb) {
            return false;
        }
        if (ra.r === rb.r && ra.g === rb.g && ra.b === rb.b) {
            return true;
        }
        ha = rgbToHsv(ra.r, ra.g, ra.b);
        hb = rgbToHsv(rb.r, rb.g, rb.b);
        greyA = ha.s < SAT_CHROMA;
        greyB = hb.s < SAT_CHROMA;
        if (greyA && greyB) {
            return Math.abs(ha.v - hb.v) < GREY_V_BLOCK;
        }
        if (greyA || greyB) {
            return false;
        }
        return hueDist(ha.h, hb.h) < HUE_BLOCK_DEG;
    }

    function takenMeta(hexes) {
        var out = [];
        var i;
        var hsv;
        for (i = 0; i < hexes.length; i++) {
            hsv = hsvOfHex(hexes[i]);
            if (!hsv) {
                continue;
            }
            out.push({
                hex: hexes[i],
                h: hsv.h,
                s: hsv.s,
                v: hsv.v,
                grey: hsv.s < SAT_CHROMA
            });
        }
        return out;
    }

    function hsvDenied(h, s, v, meta) {
        var i;
        var t;
        var grey = s < SAT_CHROMA;
        for (i = 0; i < meta.length; i++) {
            t = meta[i];
            if (grey && t.grey) {
                if (Math.abs(v - t.v) < GREY_V_BLOCK) {
                    return true;
                }
            } else if (!grey && !t.grey) {
                if (hueDist(h, t.h) < HUE_BLOCK_DEG) {
                    return true;
                }
            }
        }
        return false;
    }

    function hueDenied(h, meta) {
        return hsvDenied(h, 1, 1, meta);
    }

    function makeHatch(stroke) {
        var c = document.createElement('canvas');
        var g;
        c.width = 8;
        c.height = 8;
        g = c.getContext('2d');
        g.strokeStyle = stroke;
        g.lineWidth = 1;
        g.beginPath();
        g.moveTo(-1, 5);
        g.lineTo(5, -1);
        g.moveTo(3, 9);
        g.lineTo(9, 3);
        g.stroke();
        return c;
    }

    function hatchCanvas() {
        if (isAuroraTheme()) {
            if (!hatchAurora) {
                hatchAurora = makeHatch('rgba(58, 160, 255, 0.55)');
            }
            return hatchAurora;
        }
        if (!hatchClassic) {
            hatchClassic = makeHatch('rgba(196, 168, 88, 0.55)');
        }
        return hatchClassic;
    }

    function paintBlockout(ctx, rects) {
        var i;
        var r;
        var pattern;
        if (!ctx || !rects || !rects.length) {
            return;
        }
        ctx.save();
        ctx.fillStyle = 'rgba(6, 10, 16, 0.78)';
        for (i = 0; i < rects.length; i++) {
            r = rects[i];
            ctx.fillRect(r.x, r.y, r.w, r.h);
        }
        pattern = ctx.createPattern(hatchCanvas(), 'repeat');
        if (pattern) {
            ctx.globalAlpha = 0.42;
            ctx.fillStyle = pattern;
            for (i = 0; i < rects.length; i++) {
                r = rects[i];
                ctx.fillRect(r.x, r.y, r.w, r.h);
            }
        }
        ctx.restore();
    }

    function deniedHueRects() {
        var rects = [];
        var inRun = false;
        var runStart = 0;
        var x;
        var h;
        for (x = 0; x < HUE_W; x++) {
            h = (x / (HUE_W - 1)) * 360;
            if (hueDenied(h, taken)) {
                if (!inRun) {
                    inRun = true;
                    runStart = x;
                }
            } else if (inRun) {
                rects.push({ x: runStart, y: 0, w: x - runStart, h: HUE_H });
                inRun = false;
            }
        }
        if (inRun) {
            rects.push({ x: runStart, y: 0, w: HUE_W - runStart, h: HUE_H });
        }
        return rects;
    }

    function deniedSvRects() {
        var rects = [];
        var satPx;
        var gi;
        var t;
        var y0;
        var y1;
        if (hueDenied(state.h, taken)) {
            return [{ x: 0, y: 0, w: SV_W, h: SV_H }];
        }
        satPx = Math.round(SAT_CHROMA * (SV_W - 1));
        for (gi = 0; gi < taken.length; gi++) {
            t = taken[gi];
            if (!t.grey) {
                continue;
            }
            y0 = (1 - (t.v + GREY_V_BLOCK)) * (SV_H - 1);
            y1 = (1 - (t.v - GREY_V_BLOCK)) * (SV_H - 1);
            if (y0 < 0) {
                y0 = 0;
            }
            if (y1 > SV_H) {
                y1 = SV_H;
            }
            if (y1 > y0) {
                rects.push({ x: 0, y: y0, w: satPx + 1, h: y1 - y0 });
            }
        }
        return rects;
    }

    function currentHex() {
        var rgb = hsvToRgb(state.h, state.s, state.v);
        return rgbToHex(rgb.r, rgb.g, rgb.b);
    }

    function currentDenied() {
        return hsvDenied(state.h, state.s, state.v, taken);
    }

    function themePanelClass() {
        return isAuroraTheme() ? 'au-lobby__hue-picker' : 'cc-lobby__hue-picker';
    }

    function ensurePanel() {
        var head;
        var title;
        var closeBtn;
        var svWrap;
        var hueWrap;
        var foot;
        var hint;
        if (panel) {
            return panel;
        }

        panel = document.createElement('div');
        panel.className = 'cnc-hue-picker ' + themePanelClass();
        panel.style.display = 'none';

        head = document.createElement('div');
        head.className = 'cnc-hue-picker__head';
        title = document.createElement('span');
        title.appendChild(document.createTextNode('Custom colour'));
        closeBtn = document.createElement('button');
        closeBtn.type = 'button';
        closeBtn.className = 'cnc-hue-picker__close';
        closeBtn.setAttribute('aria-label', 'Close');
        closeBtn.appendChild(document.createTextNode('\u00d7'));
        $(closeBtn).on('click', function (e) {
            e.preventDefault();
            e.stopPropagation();
            closePicker();
        });
        head.appendChild(closeBtn);
        head.appendChild(title);

        svWrap = document.createElement('div');
        svWrap.className = 'cnc-hue-picker__sv-wrap';
        svCanvas = document.createElement('canvas');
        svCanvas.className = 'cnc-hue-picker__sv';
        svCanvas.width = SV_W;
        svCanvas.height = SV_H;
        svWrap.appendChild(svCanvas);

        hueWrap = document.createElement('div');
        hueWrap.className = 'cnc-hue-picker__hue-wrap';
        hueCanvas = document.createElement('canvas');
        hueCanvas.className = 'cnc-hue-picker__hue';
        hueCanvas.width = HUE_W;
        hueCanvas.height = HUE_H;
        hueWrap.appendChild(hueCanvas);

        foot = document.createElement('div');
        foot.className = 'cnc-hue-picker__foot';
        chipEl = document.createElement('span');
        chipEl.className = 'cnc-hue-picker__chip';
        hexEl = document.createElement('span');
        hexEl.className = 'cnc-hue-picker__hex';
        foot.appendChild(chipEl);
        foot.appendChild(hexEl);

        hint = document.createElement('div');
        hint.className = 'cnc-hue-picker__hint';
        hintEl = hint;

        panel.appendChild(head);
        panel.appendChild(svWrap);
        panel.appendChild(hueWrap);
        panel.appendChild(foot);
        panel.appendChild(hint);
        document.body.appendChild(panel);

        $(panel).on('mousedown click', function (e) {
            e.stopPropagation();
        });
        $(svCanvas).on('mousedown', function (e) {
            e.preventDefault();
            e.stopPropagation();
            startDrag('sv', e);
        });
        $(hueCanvas).on('mousedown', function (e) {
            e.preventDefault();
            e.stopPropagation();
            startDrag('hue', e);
        });
        $(svCanvas).on('mousemove', function (e) {
            if (!drag) {
                hoverAt('sv', e);
            }
        });
        $(hueCanvas).on('mousemove', function (e) {
            if (!drag) {
                hoverAt('hue', e);
            }
        });

        $(document).on('mousemove.cncHuePicker', function (e) {
            if (drag) {
                hoverAt(drag, e);
            }
        });
        $(document).on('mouseup.cncHuePicker', function () {
            if (drag) {
                finishDrag();
            }
        });
        $(document).on('mousedown.cncHuePicker', function (e) {
            var t;
            if (!panel || panel.style.display === 'none') {
                return;
            }
            t = e.target;
            if (panel.contains(t)) {
                return;
            }
            if (hostEl && (hostEl === t || (hostEl.contains && hostEl.contains(t)))) {
                return;
            }
            closePicker();
        });
        $(window).on('resize.cncHuePicker scroll.cncHuePicker', function () {
            if (panel && panel.style.display === 'block') {
                placePanel();
            }
        });

        return panel;
    }

    function canvasPos(canvas, evt) {
        var $c = $(canvas);
        var off = $c.offset() || { left: 0, top: 0 };
        var cssW = $c.outerWidth() || canvas.width;
        var cssH = $c.outerHeight() || canvas.height;
        var x = (evt.pageX - off.left) * (canvas.width / cssW);
        var y = (evt.pageY - off.top) * (canvas.height / cssH);
        if (x < 0) {
            x = 0;
        } else if (x > canvas.width - 1) {
            x = canvas.width - 1;
        }
        if (y < 0) {
            y = 0;
        } else if (y > canvas.height - 1) {
            y = canvas.height - 1;
        }
        return { x: x, y: y };
    }

    function applySvPos(pos) {
        var s = pos.x / (SV_W - 1);
        var v = 1 - (pos.y / (SV_H - 1));
        if (s < 0) {
            s = 0;
        } else if (s > 1) {
            s = 1;
        }
        if (v < 0) {
            v = 0;
        } else if (v > 1) {
            v = 1;
        }
        state.s = s;
        state.v = v;
    }

    function applyHuePos(pos) {
        var h = (pos.x / (HUE_W - 1)) * 360;
        if (h < 0) {
            h = 0;
        } else if (h >= 360) {
            h = 359.99;
        }
        state.h = h;
    }

    function hoverAt(which, evt) {
        var pos;
        if (which === 'sv') {
            pos = canvasPos(svCanvas, evt);
            applySvPos(pos);
            svCanvas.style.cursor = currentDenied() ? 'not-allowed' : 'crosshair';
        } else {
            pos = canvasPos(hueCanvas, evt);
            applyHuePos(pos);
            hueCanvas.style.cursor = hueDenied(state.h, taken) ? 'not-allowed' : 'pointer';
        }
        paint();
        updateChrome(false);
    }

    function startDrag(which, evt) {
        drag = which;
        hoverAt(which, evt);
    }

    function finishDrag() {
        drag = null;
        updateChrome(true);
    }

    function drawSv() {
        var ctx = svCanvas.getContext('2d');
        var img = ctx.createImageData(SV_W, SV_H);
        var data = img.data;
        var x;
        var y;
        var i;
        var s;
        var v;
        var rgb;

        for (y = 0; y < SV_H; y++) {
            v = 1 - (y / (SV_H - 1));
            for (x = 0; x < SV_W; x++) {
                s = x / (SV_W - 1);
                rgb = hsvToRgb(state.h, s, v);
                i = (y * SV_W + x) * 4;
                data[i] = rgb.r;
                data[i + 1] = rgb.g;
                data[i + 2] = rgb.b;
                data[i + 3] = 255;
            }
        }
        ctx.putImageData(img, 0, 0);
        paintBlockout(ctx, deniedSvRects());

        x = state.s * (SV_W - 1);
        y = (1 - state.v) * (SV_H - 1);
        ctx.beginPath();
        ctx.arc(x, y, 7, 0, Math.PI * 2);
        ctx.strokeStyle = 'rgba(10, 18, 28, 0.9)';
        ctx.lineWidth = 3;
        ctx.stroke();
        ctx.beginPath();
        ctx.arc(x, y, 6, 0, Math.PI * 2);
        ctx.strokeStyle = currentDenied() ? '#e74c3c' : '#ffffff';
        ctx.lineWidth = 2;
        ctx.stroke();
    }

    function drawHue() {
        var ctx = hueCanvas.getContext('2d');
        var x;
        var rgb;
        var h;
        var mx;

        for (x = 0; x < HUE_W; x++) {
            h = (x / (HUE_W - 1)) * 360;
            rgb = hsvToRgb(h, 1, 1);
            ctx.fillStyle = 'rgb(' + rgb.r + ',' + rgb.g + ',' + rgb.b + ')';
            ctx.fillRect(x, 0, 1, HUE_H);
        }
        paintBlockout(ctx, deniedHueRects());

        mx = (state.h / 360) * (HUE_W - 1);
        ctx.fillStyle = 'rgba(10, 18, 28, 0.95)';
        ctx.fillRect(mx - 2, 0, 4, HUE_H);
        ctx.fillStyle = hueDenied(state.h, taken) ? '#e74c3c' : '#ffffff';
        ctx.fillRect(mx - 1, 1, 2, HUE_H - 2);
    }

    function paint() {
        if (!svCanvas || !hueCanvas) {
            return;
        }
        drawSv();
        drawHue();
    }

    function updateChrome(commit) {
        var hex = currentHex();
        var denied = currentDenied();
        if (chipEl) {
            chipEl.style.backgroundColor = hex;
            chipEl.className = 'cnc-hue-picker__chip' + (denied ? ' is-denied' : '');
        }
        if (hexEl) {
            hexEl.textContent = hex.toUpperCase();
            hexEl.className = 'cnc-hue-picker__hex' + (denied ? ' is-denied' : '');
        }
        if (hintEl) {
            hintEl.textContent = denied ? 'Hue taken' : '';
        }
        if (commit && !denied && typeof onPick === 'function') {
            onPick(hex);
        }
    }

    function placePanel() {
        var $host;
        var off;
        var hw;
        var hh;
        var $p;
        var pw;
        var ph;
        var winW;
        var winH;
        var left;
        var top;
        var maxLeft;
        if (!panel || !hostEl) {
            return;
        }
        $host = $(hostEl);
        off = $host.offset() || { left: 8, top: 8 };
        hw = $host.outerWidth() || 14;
        hh = $host.outerHeight() || 12;
        $p = $(panel);
        pw = $p.outerWidth() || 188;
        ph = $p.outerHeight() || 200;
        winW = $(window).width() || 1024;
        winH = $(window).height() || 768;
        left = off.left + hw + 8;
        top = off.top;
        if (left + pw > winW - 6) {
            left = off.left - pw - 8;
        }
        if (left < 6) {
            left = 6;
        }
        maxLeft = winW - pw - 6;
        if (maxLeft < 6) {
            maxLeft = 6;
        }
        if (left > maxLeft) {
            left = maxLeft;
        }
        if (top + ph > winH - 6) {
            top = winH - ph - 6;
        }
        if (top < 6) {
            top = 6;
        }
        panel.style.left = left + 'px';
        panel.style.top = top + 'px';
    }

    function hexToState(hex) {
        var hsv = hsvOfHex(hex);
        if (!hsv) {
            state.h = 210;
            state.s = 0.72;
            state.v = 0.84;
            return;
        }
        state.h = hsv.h;
        state.s = hsv.s;
        state.v = hsv.v;
    }

    function openPicker(host, opts) {
        var list;
        opts = opts || {};
        ensurePanel();
        hostEl = host || null;
        onPick = opts.onPick || null;
        list = opts.taken || [];
        taken = takenMeta(list);
        hexToState(opts.current);
        panel.className = 'cnc-hue-picker ' + themePanelClass();
        if (window.CncLobbyTooltips && window.CncLobbyTooltips.hide) {
            window.CncLobbyTooltips.hide();
        }
        panel.style.display = 'block';
        panel.style.visibility = 'hidden';
        paint();
        updateChrome(false);
        placePanel();
        panel.style.visibility = 'visible';
        if (panel.offsetHeight) {
            /* reflow for enter animation */
        }
        panel.className = 'cnc-hue-picker ' + themePanelClass() + ' cnc-hue-picker--in';
    }

    function closePicker() {
        drag = null;
        hostEl = null;
        onPick = null;
        taken = [];
        if (panel) {
            panel.style.display = 'none';
            panel.style.visibility = 'hidden';
            panel.className = 'cnc-hue-picker ' + themePanelClass();
        }
    }

    window.CncLobbyColorPicker = {
        open: openPicker,
        close: closePicker,
        isOpen: function () {
            return !!(panel && panel.style.display === 'block');
        },
        isOpenFor: function (el) {
            return !!(panel && panel.style.display === 'block' && hostEl === el);
        },
        tooClose: tooClose,
        HUE_BLOCK_DEG: HUE_BLOCK_DEG
    };
}(window, window.jQuery));
