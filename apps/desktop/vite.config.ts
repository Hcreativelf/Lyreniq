import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { cacheDirectory } from "./cache-paths";

export default defineConfig({
  plugins: [react()],
  cacheDir: cacheDirectory("vite"),
  clearScreen: false,
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: { target: "chrome105" },
});
