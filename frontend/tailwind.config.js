import defaultTheme from 'tailwindcss/defaultTheme';
import forms from '@tailwindcss/forms';

/** @type {import('tailwindcss').Config} */
export default {
    content: [
        './vendor/laravel/framework/src/Illuminate/Pagination/resources/views/*.blade.php',
        './storage/framework/views/*.php',
        './resources/views/**/*.blade.php',
    ],

    theme: {
        extend: {
            fontFamily: {
                sans: ['Sora', ...defaultTheme.fontFamily.sans],
                display: ['Space Grotesk', ...defaultTheme.fontFamily.sans],
            },
            colors: {
                grit: {
                    bg: '#0a0b0d',
                    surface: '#121417',
                    panel: '#171a1f',
                    line: '#2a3038',
                    mist: '#9aa3ad',
                    text: '#e8edf2',
                },
                signal: {
                    DEFAULT: '#0088ff',
                    soft: '#3aa0ff',
                    mute: 'rgba(0, 136, 255, 0.14)',
                },
            },
            keyframes: {
                rise: {
                    '0%': { opacity: '0', transform: 'translateY(14px)' },
                    '100%': { opacity: '1', transform: 'translateY(0)' },
                },
                fade: {
                    '0%': { opacity: '0' },
                    '100%': { opacity: '1' },
                },
            },
            animation: {
                rise: 'rise 0.9s cubic-bezier(0.22, 1, 0.36, 1) both',
                'rise-2': 'rise 0.9s cubic-bezier(0.22, 1, 0.36, 1) 0.12s both',
                'rise-3': 'rise 1s cubic-bezier(0.22, 1, 0.36, 1) 0.22s both',
                fade: 'fade 1.1s ease both',
            },
        },
    },

    plugins: [forms],
};
