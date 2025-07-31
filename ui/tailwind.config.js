/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{js,jsx,ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        polkadot: {
          500: '#e6007a',
          600: '#d1006d',
        },
        ethereum: {
          500: '#3c4ed8',
          600: '#2938b8',
        }
      }
    },
  },
  plugins: [],
}
