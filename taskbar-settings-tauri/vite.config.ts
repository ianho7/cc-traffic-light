import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { paraglideVitePlugin } from "@inlang/paraglide-js";

export default defineConfig({
  plugins: [
    react(),
    paraglideVitePlugin({
      project: "./project.inlang",
      outdir: "./src/paraglide"
    })
  ],
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true
  }
});
