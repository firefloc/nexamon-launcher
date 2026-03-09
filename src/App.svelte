<script lang="ts">
  import { onMount } from "svelte";
  import { account, checkSavedAccount } from "./lib/stores/auth";
  import { loadSettings } from "./lib/stores/settings";
  import { loadProfiles } from "./lib/stores/profiles";
  import { listen } from "@tauri-apps/api/event";
  import { launcherState, progressInfo, addLogLine } from "./lib/stores/launcher";
  import { checkForUpdates } from "./lib/stores/updater";
  import { devMode, checkDevMode } from "./lib/stores/dev";
  import Login from "./pages/Login.svelte";
  import Main from "./pages/Main.svelte";
  import Settings from "./pages/Settings.svelte";
  import Console from "./pages/Console.svelte";
  import Dev from "./pages/Dev.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import UpdateBanner from "./components/UpdateBanner.svelte";

  let page = $state<"main" | "settings" | "console" | "dev">("main");
  let loggedIn = $derived($account !== null);

  onMount(async () => {
    await loadSettings();
    await loadProfiles();
    await checkSavedAccount();
    await checkDevMode();

    listen<{ label: string; detail: string; progress: number }>("progress", (e) => {
      progressInfo.set(e.payload);
    });

    listen<{ state: string }>("launcher_state", (e) => {
      launcherState.set(e.payload.state as any);
    });

    listen<{ line: string }>("game_log", (e) => {
      addLogLine(e.payload.line);
    });

    // Check for updates after startup
    checkForUpdates();
  });
</script>

{#if !loggedIn}
  <Login />
{:else}
  <div class="layout">
    <Sidebar bind:page devMode={$devMode} />
    <div class="main-area">
      <UpdateBanner />
      <div class="content">
        {#if page === "main"}
          <Main />
        {:else if page === "settings"}
          <Settings />
        {:else if page === "console"}
          <Console />
        {:else if page === "dev"}
          <Dev />
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .layout {
    display: flex;
    height: 100%;
    width: 100%;
  }
  .main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }
</style>
