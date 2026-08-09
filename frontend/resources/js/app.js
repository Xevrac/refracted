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

const boot = () => {
    initHeroSlider();
    initReveals();
    initSortables();
};

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
} else {
    boot();
}
