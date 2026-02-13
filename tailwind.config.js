/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'brand-purple': '#54449cff',
        'brand-purple-dark': '#764ba2',
      },
    },
  },
  safelist: [
    // 确保动态类不被清除
    'text-center',
    'text-left',
    'hover:bg-[#f8f9fa]',
  ],
  plugins: [],
}
