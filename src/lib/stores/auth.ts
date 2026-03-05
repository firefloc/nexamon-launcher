import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface Account {
  username: string;
  uuid: string;
  skin_url: string | null;
}

export interface DeviceCode {
  user_code: string;
  verification_uri: string;
  expires_in: number;
}

export const account = writable<Account | null>(null);
export const isLoggingIn = writable(false);

export async function checkSavedAccount() {
  try {
    const acc = await invoke<Account | null>("get_account");
    account.set(acc);
  } catch {
    account.set(null);
  }
}

export async function startLogin(): Promise<DeviceCode> {
  isLoggingIn.set(true);
  return await invoke<DeviceCode>("start_login");
}

export async function pollLogin(): Promise<Account> {
  try {
    const acc = await invoke<Account>("poll_login");
    account.set(acc);
    return acc;
  } finally {
    isLoggingIn.set(false);
  }
}

export async function logout() {
  await invoke("logout");
  account.set(null);
}
