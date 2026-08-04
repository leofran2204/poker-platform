/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        felt: {
          950: "#0a1f0a",
          900: "#0f2a0f",
          850: "#132f13",
          800: "#1a3a1a",
          700: "#1e4a1e",
          600: "#2d5a2d",
          500: "#3a6b3a",
          400: "#5a8a5a",
          300: "#7ab87a",
          200: "#a8d0a8",
        },
        rail: {
          DEFAULT: "#8b6914",
          light: "#a07a20",
          dark: "#6b5010",
        },
        gold: {
          DEFAULT: "#c9a227",
          bright: "#d4a843",
          dim: "#8b6914",
          soft: "#e8d48b",
        },
        cream: {
          DEFAULT: "#e8e0d0",
          muted: "#c4bba8",
        },
        ink: {
          DEFAULT: "#0c0c0c",
          panel: "#121812",
        },
      },
      fontFamily: {
        // Classic client feel without looking generic “AI SaaS”
        ui: [
          "Segoe UI",
          "Tahoma",
          "Geneva",
          "Verdana",
          "system-ui",
          "sans-serif",
        ],
        mono: ["Consolas", "ui-monospace", "monospace"],
      },
      boxShadow: {
        panel: "0 2px 0 rgba(0,0,0,0.35), 0 8px 24px rgba(0,0,0,0.35)",
        seat: "0 2px 8px rgba(0,0,0,0.45)",
        card: "0 2px 6px rgba(0,0,0,0.4)",
      },
      borderRadius: {
        table: "50%",
      },
    },
  },
  plugins: [],
};
