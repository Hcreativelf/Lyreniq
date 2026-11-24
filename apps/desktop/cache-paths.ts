import { existsSync, readFileSync } from "node:fs";
import { isAbsolute, join } from "node:path";
import { fileURLToPath } from "node:url";

// Machine preferences stay untracked; clones and CI use project-local caches.
export function cacheDirectory(tool: "vite" | "vitest") {
  const file = fileURLToPath(new URL("../../.local/cache-paths.json", import.meta.url));
  const settings = existsSync(file) ? JSON.parse(readFileSync(file, "utf8")) : {};
  const cacheRoot = process.env.LYRENIQ_CACHE_ROOT ?? settings.cacheRoot;
  if (!cacheRoot) return fileURLToPath(new URL(`../../.local/cache/${tool}`, import.meta.url));
  if (typeof cacheRoot !== "string" || !isAbsolute(cacheRoot)) {
    throw new Error("LYRENIQ_CACHE_ROOT/cacheRoot must be an absolute local cache directory");
  }
  return join(cacheRoot, tool, "Lyreniq");
}
