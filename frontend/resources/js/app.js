import Sortable from 'sortablejs';

const reduceMotion = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches;

const csrfToken = () => document.querySelector('meta[name="csrf-token"]')?.getAttribute('content') || '';

const initHeroSlider = () => {
    const root = document.querySelector('[data-ref-slider]');

    if (! root) {
        return;
    }

    const slides = Array.from(root.querySelectorAll('[data-ref-slide]'));
    const dots = Array.from(root.querySelectorAll('[data-ref-dot]'));

    if (slides.length < 2) {
        slides[0]?.classList.add('is-active');
        return;
    }

    let index = 0;
    let timer = null;

    const show = (next) => {
        const current = slides[index];
        const upcoming = slides[next];

        if (! upcoming || current === upcoming) {
            return;
        }

        current.classList.remove('is-active');
        current.classList.add('is-exit');
        current.setAttribute('aria-hidden', 'true');

        upcoming.classList.remove('is-exit');
        upcoming.classList.add('is-active');
        upcoming.setAttribute('aria-hidden', 'false');

        dots.forEach((dot, i) => {
            const active = i === next;
            dot.setAttribute('aria-current', active ? 'true' : 'false');
            dot.classList.toggle('bg-signal', active);
            dot.classList.toggle('bg-grit-line', ! active);
        });

        window.setTimeout(() => {
            current.classList.remove('is-exit');
        }, reduceMotion() ? 0 : 550);

        index = next;
    };

    const next = () => show((index + 1) % slides.length);

    const start = () => {
        stop();
        if (reduceMotion()) {
            return;
        }
        timer = window.setInterval(next, 4800);
    };

    const stop = () => {
        if (timer) {
            window.clearInterval(timer);
            timer = null;
        }
    };

    slides.forEach((slide, i) => {
        slide.classList.toggle('is-active', i === 0);
    });

    dots.forEach((dot, i) => {
        dot.addEventListener('click', () => {
            show(i);
            start();
        });
    });

    root.addEventListener('mouseenter', stop);
    root.addEventListener('mouseleave', start);
    root.addEventListener('focusin', stop);
    root.addEventListener('focusout', start);

    start();
};

const initReveals = () => {
    const nodes = Array.from(document.querySelectorAll('[data-ref-reveal]'));

    if (! nodes.length) {
        return;
    }

    if (reduceMotion()) {
        nodes.forEach((node) => node.classList.add('is-visible'));
        return;
    }

    const observer = new IntersectionObserver((entries) => {
        entries.forEach((entry) => {
            if (! entry.isIntersecting) {
                return;
            }

            entry.target.classList.add('is-visible');
            observer.unobserve(entry.target);
        });
    }, {
        threshold: 0.2,
    });

    nodes.forEach((node) => observer.observe(node));
};

const collectOrder = (root) => Array.from(root.querySelectorAll('[data-id]'))
    .map((item) => Number(item.dataset.id))
    .filter((id) => Number.isInteger(id));

const persistOrder = async (url, order) => {
    const response = await fetch(url, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            Accept: 'application/json',
            'X-CSRF-TOKEN': csrfToken(),
            'X-Requested-With': 'XMLHttpRequest',
        },
        body: JSON.stringify({ order }),
    });

    if (! response.ok) {
        throw new Error('Reorder failed');
    }
};

const initSortables = () => {
    document.querySelectorAll('[data-ref-sortable]').forEach((root) => {
        const url = root.dataset.reorderUrl;

        if (! url || root.querySelectorAll('[data-id]').length < 2) {
            return;
        }

        Sortable.create(root, {
            handle: '.ref-drag-handle',
            animation: 150,
            ghostClass: 'opacity-40',
            dragClass: 'shadow-2xl',
            onEnd: async () => {
                try {
                    await persistOrder(url, collectOrder(root));
                } catch {
                    window.location.reload();
                }
            },
        });
    });
};

const initSentryInbox = () => {
    const root = document.querySelector('[data-sentry-bell]');

    if (! root) {
        return;
    }

    const url = root.dataset.inboxUrl;
    const count = root.querySelector('[data-bell-count]');
    const headingCount = root.querySelector('[data-bell-heading-count]');
    const markAll = root.querySelector('[data-mark-all]');
    const toggle = root.querySelector('[data-bell-toggle]');
    const menu = root.querySelector('[data-bell-menu]');
    const list = root.querySelector('[data-bell-list]');
    const empty = root.querySelector('[data-bell-empty]');
    const table = document.querySelector('#sentry-issues');
    let seen = null;
    let open = false;

    const headers = {
        Accept: 'application/json',
        'Content-Type': 'application/json',
        'X-CSRF-TOKEN': csrfToken(),
        'X-Requested-With': 'XMLHttpRequest',
    };

    const setCount = (value) => {
        const next = Number(value) || 0;
        const label = next > 99 ? '99+' : String(next);
        count.hidden = next < 1;
        count.textContent = label;
        if (headingCount) {
            headingCount.textContent = next > 0 ? label : '';
        }
        if (markAll) {
            markAll.disabled = next < 1;
        }
        const base = document.title.replace(/^\(\d+\)\s*/, '');
        document.title = next > 0 ? `(${next}) ${base}` : base;

        if (seen !== null && next > seen) {
            toggle.classList.add('is-alert');
            window.setTimeout(() => toggle.classList.remove('is-alert'), 700);
        }

        seen = next;
    };

    const placeDot = (issue) => {
        const row = document.querySelector(`[data-issue-id="${issue.id}"]`);
        const cell = row?.querySelector('.ref-issue-id');

        if (! cell || cell.querySelector('[data-read-url]')) {
            return;
        }

        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'ref-unread';
        button.dataset.readUrl = issue.read_url;
        button.setAttribute('aria-label', 'Mark as read');
        cell.prepend(button);
    };

    const matches = (issue) => {
        if (! table || table.dataset.page !== '1') {
            return false;
        }

        const status = table.dataset.status || 'unresolved';
        const game = table.dataset.game || '';
        const type = table.dataset.type || '';
        const query = (table.dataset.query || '').toLowerCase();

        if (status !== 'all' && issue.status !== status) {
            return false;
        }

        if (game && issue.game_slug !== game) {
            return false;
        }

        if (type && issue.type !== type) {
            return false;
        }

        return query === '' || String(issue.title).toLowerCase().includes(query);
    };

    const cell = (text, className) => {
        const td = document.createElement('td');
        td.className = className;
        td.textContent = text;
        return td;
    };

    const insertRow = (issue) => {
        if (! table || document.querySelector(`[data-issue-id="${issue.id}"]`) || ! matches(issue)) {
            return;
        }

        table.querySelector('[data-empty]')?.remove();

        const row = document.createElement('tr');
        row.className = 'border-b border-grit-line last:border-b-0 hover:bg-signal/10';
        row.dataset.issueId = String(issue.id);

        const idCell = document.createElement('td');
        idCell.className = 'ref-issue-id px-4 py-3 font-mono text-xs text-grit-mist';
        const dot = document.createElement('button');
        dot.type = 'button';
        dot.className = 'ref-unread';
        dot.dataset.readUrl = issue.read_url;
        dot.setAttribute('aria-label', 'Mark as read');
        const link = document.createElement('a');
        link.href = issue.url;
        link.textContent = `#${issue.id}`;
        idCell.append(dot, link);

        const titleCell = document.createElement('td');
        titleCell.className = 'max-w-md px-4 py-3';
        const title = document.createElement('a');
        title.href = issue.url;
        title.className = 'block truncate font-medium text-grit-text hover:text-signal';
        title.textContent = issue.title;
        titleCell.append(title);

        const statusCell = cell(issue.status_label, `px-4 py-3 text-xs uppercase tracking-[0.08em] ${issue.status === 'unresolved' ? 'text-signal' : 'text-grit-mist'}`);
        statusCell.dataset.issueStatus = issue.status;

        row.append(
            idCell,
            cell(issue.game, 'px-4 py-3 text-xs text-grit-mist'),
            cell(issue.type, 'px-4 py-3 text-xs uppercase tracking-[0.08em] text-grit-mist'),
            titleCell,
            statusCell,
            cell(String(issue.events), 'px-4 py-3 text-right tabular-nums'),
            cell(issue.seen, 'px-4 py-3 text-right text-xs text-grit-mist'),
        );
        table.prepend(row);
    };

    const paintMenu = (issues) => {
        list.replaceChildren();

        if (! issues.length) {
            const note = document.createElement('p');
            note.className = 'px-4 py-5 text-sm text-grit-mist';
            note.dataset.bellEmpty = '';
            note.textContent = 'No unread incidents.';
            list.append(note);
            return;
        }

        issues.forEach((issue) => {
            const item = document.createElement('a');
            item.href = issue.url;
            item.className = 'ref-bell-item';

            const dot = document.createElement('span');
            dot.className = 'ref-bell-dot';
            dot.setAttribute('aria-hidden', 'true');

            const body = document.createElement('span');
            body.className = 'ref-bell-copy';
            const title = document.createElement('span');
            title.className = 'ref-bell-title';
            title.textContent = issue.title;

            const meta = document.createElement('span');
            meta.className = 'ref-bell-meta';
            const where = document.createElement('span');
            where.textContent = `${issue.game} · ${issue.type}`;
            const when = document.createElement('span');
            when.textContent = issue.seen;
            meta.append(where, when);
            body.append(title, meta);
            item.append(dot, body);
            list.append(item);
        });
    };

    const refresh = async () => {
        const response = await fetch(url, { headers, credentials: 'same-origin' });

        if (! response.ok) {
            return;
        }

        const body = await response.json();
        const issues = Array.isArray(body.issues) ? body.issues : [];
        const ids = new Set((Array.isArray(body.ids) ? body.ids : issues.map((issue) => issue.id)).map((id) => String(id)));

        setCount(body.unread);
        paintMenu(issues);
        const readTemplate = issues[0]?.read_url || '';
        document.querySelectorAll('[data-issue-id]').forEach((row) => {
            if (! ids.has(row.dataset.issueId)) {
                row.querySelector('[data-read-url]')?.remove();
                return;
            }

            if (! row.querySelector('[data-read-url]') && readTemplate) {
                placeDot({
                    id: row.dataset.issueId,
                    read_url: readTemplate.replace(/\/\d+\/read$/, `/${row.dataset.issueId}/read`),
                });
            }
        });
        issues.forEach((issue) => {
            const row = document.querySelector(`[data-issue-id="${issue.id}"]`);

            if (row && ! matches(issue)) {
                row.remove();
                return;
            }

            if (row) {
                const statusCell = row.querySelector('[data-issue-status]');

                if (statusCell) {
                    statusCell.textContent = issue.status_label;
                    statusCell.dataset.issueStatus = issue.status;
                    statusCell.className = `px-4 py-3 text-xs uppercase tracking-[0.08em] ${issue.status === 'unresolved' ? 'text-signal' : 'text-grit-mist'}`;
                }
            }

            insertRow(issue);
            placeDot(issue);
        });
    };

    document.addEventListener('click', async (event) => {
        const button = event.target.closest('[data-read-url]');

        if (! button) {
            return;
        }

        event.preventDefault();
        event.stopPropagation();

        const response = await fetch(button.dataset.readUrl, {
            method: 'POST',
            headers,
            credentials: 'same-origin',
        });

        if (! response.ok) {
            return;
        }

        button.remove();
        const body = await response.json();
        setCount(body.unread);
        refresh();
    });

    toggle.addEventListener('click', (event) => {
        event.stopPropagation();
        open = ! open;
        menu.hidden = ! open;
        toggle.setAttribute('aria-expanded', open ? 'true' : 'false');
    });

    markAll?.addEventListener('click', async (event) => {
        event.preventDefault();
        event.stopPropagation();

        if (markAll.disabled) {
            return;
        }

        const response = await fetch(markAll.dataset.markAll, {
            method: 'POST',
            headers,
            credentials: 'same-origin',
        });

        if (! response.ok) {
            return;
        }

        const body = await response.json();
        setCount(body.unread);
        document.querySelectorAll('[data-read-url]').forEach((dot) => dot.remove());
        refresh();
    });

    document.addEventListener('click', (event) => {
        if (! root.contains(event.target)) {
            open = false;
            menu.hidden = true;
            toggle.setAttribute('aria-expanded', 'false');
        }
    });

    if (empty) {
        empty.hidden = false;
    }

    refresh();
    window.setInterval(() => {
        refresh();
    }, 4000);
};

const boot = () => {
    initHeroSlider();
    initReveals();
    initSortables();
    initSentryInbox();
};

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
} else {
    boot();
}
