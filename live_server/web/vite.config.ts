import { defineConfig } from "vite";

export default defineConfig({
  // Dev: proxy API routes to the Rust backend so the browser page and WS share an origin.
  server: {
    proxy: {
      "/ws": { target: "ws://localhost:3030", ws: true },
      "/scenes": { target: "http://localhost:3030" },
      "/resolutions": { target: "http://localhost:3030" },
      "/health": { target: "http://localhost:3030" },
    },
  },
  // Build: emit next to the Rust crate so ServeDir finds it.
  build: { outDir: "../static", emptyOutDir: true },
});
