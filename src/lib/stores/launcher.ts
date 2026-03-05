import { writable } from "svelte/store";

export type LauncherState =
  | "idle"
  | "checking_java"
  | "downloading_java"
  | "downloading_minecraft"
  | "installing_fabric"
  | "syncing_mods"
  | "launching"
  | "running";

export interface ProgressInfo {
  label: string;
  detail: string;
  progress: number; // 0.0 - 1.0
}

export const launcherState = writable<LauncherState>("idle");
export const progressInfo = writable<ProgressInfo>({
  label: "",
  detail: "",
  progress: 0,
});
export const gameLog = writable<string[]>([]);

export function addLogLine(line: string) {
  gameLog.update((lines) => {
    const next = [...lines, line];
    if (next.length > 5000) next.splice(0, next.length - 5000);
    return next;
  });
}

export function clearLog() {
  gameLog.set([]);
}
