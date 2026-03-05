<script lang="ts">
  import { onMount } from "svelte";
  import { account, checkSavedAccount } from "./lib/stores/auth";
  import { loadSettings } from "./lib/stores/settings";
  import { loadProfiles } from "./lib/stores/profiles";
  import { listen } from "@tauri-apps/api/event";
  import { launcherState, progressInfo, addLogLine } from "./lib/stores/launcher";
  import Login from "./pages/Login.svelte";
  import Main from "./pages/Main.svelte";
  import Settings from "./pages/Settings.svelte";
  import Console from "./pages/Console.svelte";
  import Sidebar from "./components/Sidebar.svelte";

  let page = $state<"main" | "settings" | "console">("main");
  let loggedIn = $derived($account !== null);

  onMount(async () => {
    await loadSettings();
    await loadProfiles();
    await checkSavedAccount();

    listen<{ label: string; detail: string; progress: number }>("progress", (e) => {
      progressInfo.set(e.payload);
    });

    listen<{ state: string }>("launcher_state", (e) => {
      launcherState.set(e.payload.state as any);
    });

    listen<{ line: string }>("game_log", (e) => {
      addLogLine(e.payload.line);
    });
  });
</script>

{#if !loggedIn}
  <Login />
{:else}
  <div class="layout">
    <Sidebar bind:page />
    <div class="content">
      {#if page === "main"}
        <Main />
      {:else if page === "settings"}
        <Settings />
      {:else if page === "console"}
        <Console />
      {/if}
    </div>
  </div>
{/if}

<style>
  .layout {
    display: flex;
    height: 100%;
    width: 100%;
  }
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }
</style>
