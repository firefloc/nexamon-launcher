import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export const devMode = writable(false);
export const debugOutlines = writable(false);

export async function checkDevMode() {
  try {
    const isDev = await invoke<boolean>("is_dev_mode");
    devMode.set(isDev);
  } catch {
    devMode.set(false);
  }
}
