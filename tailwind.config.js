module.exports = {
    content: [
        "./src/**/*.rs",
        "./index.html",
        "./src/**/*.html",
        "./src/**/*.css",
        "./index.css"
    ],
    theme: {
        extend: {
            strokeWidth: {
                '3': '3px',
                '4': '4px',
                '8': '8px',
                '12': '12px',
                '16': '16px',
            },
            zIndex: {
              '1': '1',
              '2': '2',
              '3': '3',
              '4': '4',
              '5': '5',
              '6': '6',
              '7': '7',
              '8': '8',
              '9': '9',
            },
            blur: {
                xs: '1px',
            },
            colors: {
                current: 'currentColor',

                'base': {
                    1: 'rgb(var(--color-base1) / <alpha-value>)',
                    2: 'rgb(var(--color-base2) / <alpha-value>)',
                    3: 'rgb(var(--color-base3) / <alpha-value>)',
                    4: 'rgb(var(--color-base4) / <alpha-value>)',
                    5: 'rgb(var(--color-base5) / <alpha-value>)',
                },

                'main': {
                    DEFAULT: 'rgb(var(--color-main) / <alpha-value>)',
                    ia: 'rgb(var(--color-main) / <alpha-value>)',
                },

                'red': {
                    DEFAULT: 'rgb(var(--color-red) / <alpha-value>)',
                    dark: 'rgb(var(--color-red-dark) / <alpha-value>)',
                    darker: 'rgb(var(--color-red-darker) / <alpha-value>)',
                },

                'orange': {
                    DEFAULT: 'rgb(var(--color-orange) / <alpha-value>)',
                    dark: 'rgb(var(--color-orange-dark) / <alpha-value>)',
                    darker: 'rgb(var(--color-orange-darker) / <alpha-value>)',
                },

                'yellow': {
                    DEFAULT: 'rgb(var(--color-yellow) / <alpha-value>)',
                    dark: 'rgb(var(--color-yellow-dark) / <alpha-value>)',
                    darker: 'rgb(var(--color-yellow-darker) / <alpha-value>)',
                },

                'blue': {
                    DEFAULT: 'rgb(var(--color-blue) / <alpha-value>)',
                    dark: 'rgb(var(--color-blue-dark) / <alpha-value>)',
                    darker: 'rgb(var(--color-blue-darker) / <alpha-value>)',
                },

                'green': {
                    DEFAULT: 'rgb(var(--color-green) / <alpha-value>)',
                    dark: 'rgb(var(--color-green-dark) / <alpha-value>)',
                    darker: 'rgb(var(--color-green-darker) / <alpha-value>)',
                },

                'purple': {
                    DEFAULT: 'rgb(var(--color-purple) / <alpha-value>)',
                    dark: 'rgb(var(--color-purple-dark) / <alpha-value>)',
                    darker: 'rgb(var(--color-purple-darker) / <alpha-value>)',
                },
            }
        },
    },
    variants: {},
    plugins: [],
};
