module.exports = {
    content: [
        "./src/**/*.rs",
        "./index.html",
        "./src/**/*.html",
        "./src/**/*.css",
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
            }
        }      
    },         
    variants: {},
    plugins: [],
};
