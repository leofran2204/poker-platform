import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath, URL } from "node:url";
import { stripCourseAnswerKey } from "./src/lib/courseAnswerKey";

/** O gabarito do quiz fica só na API: o site recebe as aulas sem ele. */
function courseAnswerKeyOut(): Plugin {
  return {
    name: "zt-course-answer-key-out",
    enforce: "pre",
    transform(code, id) {
      if (!id.replace(/\\/g, "/").split("?")[0].endsWith("/src/data/courseContent.json")) return null;
      return { code: JSON.stringify(stripCourseAnswerKey(JSON.parse(code))), map: null };
    },
  };
}

const apiTarget = process.env.POKER_DEV_API_URL ?? "http://127.0.0.1:3000";

export default defineConfig({
  plugins: [courseAnswerKeyOut(), react()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  server: {
    port: 5173,
    proxy: {
      "/api": {
        target: apiTarget,
        changeOrigin: true,
      },
      "/ws": {
        target: "ws://127.0.0.1:3000",
        ws: true,
      },
      "/health": {
        target: "http://127.0.0.1:3000",
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: "dist",
    sourcemap: false,
    assetsDir: "assets",
  },
});
