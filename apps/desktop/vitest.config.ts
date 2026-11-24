import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";
import { cacheDirectory } from "./cache-paths";

export default defineConfig({
  plugins: [react()],
  cacheDir: cacheDirectory("vitest"),
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.test.{ts,tsx}"],
    clearMocks: true,
  },
});
