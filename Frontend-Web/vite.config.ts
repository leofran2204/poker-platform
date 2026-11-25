import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath, URL } from "node:url";
import { publicCourseContent } from "./src/lib/coursePublicContent";

/** Gabarito do quiz e texto das aulas de aluno ficam só na API. */
function coursePublicContentOnly(): Plugin {
  return {
    name: "zt-course-public-content",
    enforce: "pre",
    transform(code, id) {
      if (!id.replace(/\\/g, "/").split("?")[0].endsWith("/src/data/courseContent.json")) return null;
      return { code: JSON.stringify(publicCourseContent(JSON.parse(code))), map: null };
    },
  };
}

const apiTarget = process.env.POKER_DEV_API_URL ?? "http://127.0.0.1:3000";

export default defineConfig({
  plugins: [coursePublicContentOnly(), react()],
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
