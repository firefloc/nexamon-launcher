import { writable } from "svelte/store";
import { check } from "@tauri-apps/plugin-updater";

export interface UpdateInfo {
  version: string;
  body: string;
}

export const updateAvailable = writable<UpdateInfo | null>(null);
export const updateProgress = writable<string>("");

let updateRef: Awaited<ReturnType<typeof check>> | null = null;

export async function checkForUpdates() {
  try {
    const update = await check();
    if (update) {
      updateRef = update;
      updateAvailable.set({
        version: update.version,
        body: update.body ?? "",
      });
    }
  } catch (e) {
    console.warn("Update check failed:", e);
  }
}

export async function installUpdate() {
  if (!updateRef) return;
  try {
    updateProgress.set("Downloading...");
    await updateRef.downloadAndInstall((event) => {
      if (event.event === "Started" && event.data.contentLength) {
        updateProgress.set(`Downloading (${Math.round(event.data.contentLength / 1024 / 1024)}MB)...`);
      } else if (event.event === "Progress") {
        updateProgress.set(`Downloading...`);
      } else if (event.event === "Finished") {
        updateProgress.set("Restarting...");
      }
    });
    // Tauri will auto-restart after install
  } catch (e) {
    console.error("Update install failed:", e);
    updateProgress.set("");
  }
}

export function dismissUpdate() {
  updateAvailable.set(null);
  updateRef = null;
}
