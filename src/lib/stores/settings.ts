import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface Settings {
  ram_mb: number;
  java_path: string | null;
  close_on_launch: boolean;
}

const defaults: Settings = {
  ram_mb: 4096,
  java_path: null,
  close_on_launch: false,
};

export const settings = writable<Settings>(defaults);

export async function loadSettings() {
  try {
    const s = await invoke<Settings>("get_settings");
    settings.set(s);
  } catch {
    settings.set(defaults);
  }
}

export async function saveSettings(s: Settings) {
  // Normalize empty java_path to null
  const normalized = { ...s, java_path: s.java_path || null };
  await invoke("save_settings", { settings: normalized });
  settings.set(normalized);
}
