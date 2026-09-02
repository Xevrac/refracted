/**
 * Lobby hover tooltips — WebKit 535 / Chrome 15 safe (no CSS attr() tips).
 * Reads data-tip / data-tip-pos on lobby controls; one shared floating label.
 * Hover: panel slides in sideways + per-character type-in stagger.
 */
(function (window, $) {
    'use strict';

    if (!$) {
        return;
    }

    var tipEl = null;
    var activeHost = null;
    var hideTimer = null;
    var lastTipKey = '';

    function isAuroraTheme() {
        return /(?:^|\s)cc-theme--aurora(?:\s|$)/.test(document.body.className || '');
    }

    function tipNode() {
        if (tipEl) {
            return tipEl;
        }
        tipEl = document.createElement('div');
        tipEl.className = 'cc-lobby__hover-tip';
        tipEl.setAttribute('aria-hidden', 'true');
        document.body.appendChild(tipEl);
        return tipEl;
    }

    function tipText(host) {
        var text = host.getAttribute('data-tip');
        return text ? String(text) : '';
    }

    function tipClass(host) {
        var base = isAuroraTheme() ? 'au-lobby__hover-tip' : 'cc-lobby__hover-tip';
        var tone = host.getAttribute('data-tip-tone');
        if (tone === 'ready') {
            return base + ' ' + base + '--ready';
        }
        if (tone === 'not-ready') {
            return base + ' ' + base + '--not-ready';
        }
        return base;
    }

    function tipDirectionClass(fromRight) {
        return fromRight ? ' shell-tip--from-right' : ' shell-tip--from-left';
    }

    function charStep(count) {
        if (count <= 1) {
            return 0;
        }
        return Math.min(46, Math.max(20, Math.round(440 / count)));
    }

    function buildAnimatedContent(el, text, fromRight, host) {
        el.innerHTML = '';
        el.className = tipClass(host) + tipDirectionClass(fromRight);

        var track = document.createElement('span');
        track.className = 'shell-tip__track';
        var chars = text.split('');
        var step = charStep(chars.length);
        var i;
        var span;
        var ch;

        for (i = 0; i < chars.length; i++) {
            ch = chars[i];
            span = document.createElement('span');
            span.className = 'shell-tip__char';
            span.style.animationDelay = (i * step) + 'ms';
            span.style.webkitAnimationDelay = (i * step) + 'ms';
            span.appendChild(document.createTextNode(ch === ' ' ? '\u00A0' : ch));
            track.appendChild(span);
        }

        el.appendChild(track);
        if (el.offsetHeight) {
            /* reflow */
        }
        el.className += ' shell-tip--in';
    }

    function placeTip(host) {
        var el = tipNode();
        var text = tipText(host);
        if (!text) {
            hideTip();
            return;
        }

        var tipKey = text + '|' + (host.getAttribute('data-tip-tone') || '');
        var reuseAnim = activeHost === host && lastTipKey === tipKey;

        var $host = $(host);
        var off = $host.offset();
        var hw = $host.outerWidth() || 0;
        var hh = $host.outerHeight() || 0;
        var winW = $(window).width() || 0;
        var fromRight = (off.left + (hw / 2)) > (winW * 0.52);

        if (!reuseAnim) {
            buildAnimatedContent(el, text, fromRight, host);
            lastTipKey = tipKey;
        } else {
            el.className = tipClass(host) + tipDirectionClass(fromRight) + ' shell-tip--in';
        }

        el.style.display = 'block';
        el.style.visibility = 'hidden';

        var tw = $(el).outerWidth() || 0;
        var th = $(el).outerHeight() || 0;
        var below = host.getAttribute('data-tip-pos') === 'below';
        var left = off.left + (hw / 2) - (tw / 2);
        var top = below ? off.top + hh + 7 : off.top - th - 7;
        var maxLeft = winW - tw - 4;

        if (maxLeft < 4) {
            maxLeft = 4;
        }
        if (left < 4) {
            left = 4;
        } else if (left > maxLeft) {
            left = maxLeft;
        }
        if (top < 4) {
            top = below ? off.top + hh + 7 : 4;
        }

        el.style.left = left + 'px';
        el.style.top = top + 'px';
        el.style.visibility = 'visible';
        activeHost = host;
    }

    function hideTip() {
        if (hideTimer) {
            clearTimeout(hideTimer);
            hideTimer = null;
        }
        if (!tipEl) {
            activeHost = null;
            lastTipKey = '';
            return;
        }
        tipEl.style.display = 'none';
        tipEl.style.visibility = 'hidden';
        tipEl.innerHTML = '';
        tipEl.className = isAuroraTheme() ? 'au-lobby__hover-tip' : 'cc-lobby__hover-tip';
        activeHost = null;
        lastTipKey = '';
    }

    function scheduleHide() {
        if (hideTimer) {
            clearTimeout(hideTimer);
        }
        hideTimer = setTimeout(hideTip, 40);
    }

    function onEnter() {
        if (hideTimer) {
            clearTimeout(hideTimer);
            hideTimer = null;
        }
        placeTip(this);
    }

    function onLeave() {
        if (activeHost === this) {
            scheduleHide();
        }
    }

    var tipSelector = '[data-tip]';

    $(document)
        .on('mouseenter', tipSelector, onEnter)
        .on('mouseleave', tipSelector, onLeave)
        .on('mousedown', tipSelector, hideTip)
        .on('scroll', '.cc-lobby, .au-lobby, .cc-shell-root, .au-dash', hideTip);

    $(window).on('resize scroll', hideTip);

    window.CncLobbyTooltips = {
        hide: hideTip,
        refresh: function () {
            if (activeHost) {
                lastTipKey = '';
                placeTip(activeHost);
            }
        }
    };
}(window, window.jQuery));
