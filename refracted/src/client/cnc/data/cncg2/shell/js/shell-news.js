/**
 * In-game news
 */
(function (window) {
    'use strict';

    var POLL_MS = 5 * 60 * 1000;
    var TIMEOUT_MS = 10000;
    var CACHE_PREFIX = 'cnc_news_v1_';
    var MAX_ARTICLES = 20;
    var MAX_BLOCKS = 40;
    var MAX_ITEMS = 40;

    var listeners = [];
    var articles = null;
    var timer = null;
    var inFlight = false;

    function config() {
        var c = window.__CNC_NEWS;
        return (c && typeof c === 'object') ? c : {};
    }

    function realm() {
        var r = config().realm;
        if (r === 'prod' || r === 'dev') {
            return r;
        }
        return window.__CNC_PLAYTEST === true ? 'dev' : 'prod';
    }

    function feedUrl(r) {
        var base = config().url;
        if (typeof base !== 'string' || !/^https?:\/\//i.test(base)) {
            return null;
        }
        var url = base.replace(/\/+$/, '') + '/api/shell/news/' + r;
        var key = config().devKey;
        if (r === 'dev' && typeof key === 'string' && key) {
            url += '?key=' + encodeURIComponent(key);
        }
        return url;
    }

    function str(v, max) {
        if (typeof v !== 'string') {
            return '';
        }
        return v.length > max ? v.substring(0, max) : v;
    }

    function httpUrl(v) {
        return (typeof v === 'string' && /^https?:\/\/[^\s"'<>]+$/i.test(v)) ? v : null;
    }

    function normalizeBlocks(blocks) {
        var out = [];
        if (!(blocks instanceof Array)) {
            return out;
        }
        for (var i = 0; i < blocks.length && out.length < MAX_BLOCKS; i++) {
            var b = blocks[i];
            if (!b || typeof b !== 'object') {
                continue;
            }
            if (b.type === 'paragraph') {
                var text = str(b.text, 4000);
                if (!text) {
                    continue;
                }
                var parts = text.split(/\n\s*\n/);
                var paragraphs = [];
                for (var p = 0; p < parts.length; p++) {
                    if (parts[p].replace(/\s+/g, '')) {
                        paragraphs.push(parts[p]);
                    }
                }
                out.push({ type: 'paragraph', paragraphs: paragraphs });
            } else if (b.type === 'heading') {
                var heading = str(b.text, 120);
                if (heading) {
                    out.push({ type: 'heading', text: heading });
                }
            } else if (b.type === 'list') {
                var items = [];
                var src = (b.items instanceof Array) ? b.items : [];
                for (var j = 0; j < src.length && items.length < MAX_ITEMS; j++) {
                    var it = src[j] || {};
                    var children = [];
                    var kids = (it.children instanceof Array) ? it.children : [];
                    for (var k = 0; k < kids.length && children.length < MAX_ITEMS; k++) {
                        var c = str(kids[k], 280);
                        if (c) {
                            children.push(c);
                        }
                    }
                    var itemText = str(it.text, 280);
                    if (itemText || children.length) {
                        items.push({ text: itemText, children: children });
                    }
                }
                if (items.length) {
                    out.push({ type: 'list', items: items });
                }
            } else if (b.type === 'divider') {
                out.push({ type: 'divider' });
            }
        }
        return out;
    }

    /** The feed document -> view models; null when it is not a schema-1 feed for this realm. */
    function normalize(doc, r) {
        if (!doc || typeof doc !== 'object' || doc.schema !== 1 || doc.realm !== r
            || !(doc.articles instanceof Array)) {
            return null;
        }
        var out = [];
        for (var i = 0; i < doc.articles.length && out.length < MAX_ARTICLES; i++) {
            var a = doc.articles[i];
            if (!a || typeof a !== 'object') {
                continue;
            }
            var title = str(a.title, 120);
            if (!title) {
                continue;
            }
            var image = httpUrl(a.image);
            out.push({
                id: (typeof a.id === 'number') ? a.id : i + 1,
                title: title,
                summary: str(a.summary, 280),
                date: str(a.date, 40),
                pinned: a.pinned === true,
                image: image,
                hasImage: !!image,
                blocks: normalizeBlocks(a.blocks)
            });
        }
        return out;
    }

    function readCache(r) {
        try {
            var raw = window.localStorage && window.localStorage.getItem(CACHE_PREFIX + r);
            return raw ? normalize(JSON.parse(raw), r) : null;
        } catch (e) {
            return null;
        }
    }

    function writeCache(r, doc) {
        try {
            if (window.localStorage) {
                window.localStorage.setItem(CACHE_PREFIX + r, JSON.stringify(doc));
            }
        } catch (e) { /* quota / disabled storage: the live feed still works */ }
    }

    function publish(list) {
        articles = list;
        for (var i = 0; i < listeners.length; i++) {
            try {
                listeners[i](articles);
            } catch (e) { /* one bad view must not stop the others */ }
        }
    }

    function poll() {
        var r = realm();
        var url = feedUrl(r);
        if (!url || inFlight || typeof XMLHttpRequest === 'undefined') {
            return;
        }
        inFlight = true;
        var xhr = new XMLHttpRequest();
        var guard = window.setTimeout(function () {
            inFlight = false;
            try { xhr.abort(); } catch (e) { /* ignore */ }
        }, TIMEOUT_MS);
        xhr.onreadystatechange = function () {
            if (xhr.readyState !== 4) {
                return;
            }
            window.clearTimeout(guard);
            inFlight = false;
            if (xhr.status !== 200) {
                return;
            }
            var doc = null;
            try {
                doc = JSON.parse(xhr.responseText);
            } catch (e) {
                return;
            }
            var list = normalize(doc, r);
            if (list) {
                writeCache(r, doc);
                publish(list);
            }
        };
        try {
            xhr.open('GET', url, true);
            xhr.send(null);
        } catch (e2) {
            window.clearTimeout(guard);
            inFlight = false;
        }
    }

    function start() {
        if (timer !== null) {
            return;
        }
        var cached = readCache(realm());
        if (cached) {
            publish(cached);
        }
        poll();
        timer = window.setInterval(poll, POLL_MS);
    }

    window.CncNews = {
        realm: realm,
        normalize: normalize,
        refresh: poll,
        /** Calls fn(articles) now (if loaded) and on every update; returns an unsubscribe. */
        subscribe: function (fn) {
            listeners.push(fn);
            start();
            if (articles) {
                fn(articles);
            }
            return function () {
                for (var i = listeners.length - 1; i >= 0; i--) {
                    if (listeners[i] === fn) {
                        listeners.splice(i, 1);
                    }
                }
            };
        }
    };
})(window);
