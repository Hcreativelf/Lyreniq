import { invoke } from "@tauri-apps/api/core";

export function readRuntimeLabel(): Promise<string> {
  return invoke<string>("runtime_label");
}
