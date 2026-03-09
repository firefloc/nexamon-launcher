import { writable, get } from "svelte/store";
import { check } from "@tauri-apps/plugin-updater";
import { t } from "../i18n";

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
    updateProgress.set(get(t)("update.downloading"));
    await updateRef.downloadAndInstall((event) => {
      if (event.event === "Started" || event.event === "Progress") {
        updateProgress.set(get(t)("update.downloading"));
      } else if (event.event === "Finished") {
        updateProgress.set(get(t)("update.restarting"));
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
