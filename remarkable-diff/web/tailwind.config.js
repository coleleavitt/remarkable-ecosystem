/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'rm-black': '#000000',
        'rm-gray': '#9d9d9d',
        'rm-white': '#ffffff',
        'rm-yellow': '#fbf432',
        'rm-green': '#00ff00',
        'rm-pink': '#f774f2',
        'rm-blue': '#0070ff',
        'rm-red': '#ff0000',
      }
    },
  },
  plugins: [],
}
