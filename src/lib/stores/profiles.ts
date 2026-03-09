import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface Profile {
  id: string;
  name: string;
  pack_url: string;
  icon: string;
  description: string;
  last_played: string | null;
}

export interface ProfilesData {
  profiles: Profile[];
  selected: string;
}

export const profiles = writable<Profile[]>([]);
export const selectedProfileId = writable<string>("");
/** Map of profile_id → "installed" | "not_installed" */
export const packStatuses = writable<Record<string, string>>({});

export async function loadProfiles() {
  try {
    const data = await invoke<ProfilesData>("get_profiles");
    profiles.set(data.profiles);
    selectedProfileId.set(data.selected);
    await refreshPackStatuses();
  } catch {
    profiles.set([]);
  }
}

export async function refreshPackStatuses() {
  try {
    const statuses = await invoke<Record<string, string>>("get_pack_statuses");
    packStatuses.set(statuses);
  } catch {
    // ignore
  }
}

export async function selectProfile(id: string) {
  await invoke("set_selected_profile", { id });
  selectedProfileId.set(id);
}

export async function addProfile(
  name: string,
  packUrl: string,
  icon: string,
  description: string,
) {
  const data = await invoke<ProfilesData>("add_profile", {
    name,
    packUrl,
    icon,
    description,
  });
  profiles.set(data.profiles);
  selectedProfileId.set(data.selected);
}

export async function removeProfile(id: string) {
  const data = await invoke<ProfilesData>("remove_profile", { id });
  profiles.set(data.profiles);
  selectedProfileId.set(data.selected);
}
