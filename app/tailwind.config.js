/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // OCPS Dark Theme — neutral greys, no blue tint
        panel: {
          bg:      "#1a1a1a",
          border:  "#2a2a2a",
          hover:   "#252525",
          active:  "#2f2f2f",
        },
        surface: {
          50:  "#f5f5f5",
          100: "#ebebeb",
          200: "#2a2a2a",
          300: "#333333",
          400: "#3d3d3d",
          500: "#474747",
          600: "#525252",
          700: "#5c5c5c",
          800: "#6b6b6b",
          900: "#7a7a7a",
        },
        accent: {
          DEFAULT: "#4a9eff",
          hover:   "#5aadff",
          muted:   "#4a9eff33",
        },
        star:    "#e8b84b",
        pick:    "#4a9eff",
        reject:  "#ff4a4a",
        label: {
          red:    "#e05252",
          yellow: "#e0c252",
          green:  "#52a052",
          blue:   "#5272e0",
          purple: "#9052e0",
        },
      },
      fontFamily: {
        sans: ["-apple-system", "BlinkMacSystemFont", "Segoe UI", "sans-serif"],
        mono: ["JetBrains Mono", "Menlo", "Monaco", "monospace"],
      },
      fontSize: {
        "2xs": "10px",
        xs:    "11px",
        sm:    "12px",
        base:  "13px",
        lg:    "14px",
      },
    },
  },
  plugins: [],
};
