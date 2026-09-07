import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        void: "#05060a",
        panel: "#0d1018",
        line: "#1c2333",
        mute: "#8b95a8",
        ink: "#e8eef7",
        phosphor: "#3ee58a",
        signal: "#4da6ff",
        warn: "#f0c14b",
        danger: "#ff5d73",
      },
      fontFamily: {
        sans: ["var(--font-sans)", "system-ui", "sans-serif"],
        mono: ["var(--font-mono)", "ui-monospace", "monospace"],
      },
      boxShadow: {
        glow: "0 0 40px rgba(62, 229, 138, 0.08)",
      },
    },
  },
  plugins: [],
};

export default config;
