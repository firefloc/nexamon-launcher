<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { profiles, selectedProfileId, selectProfile } from "../lib/stores/profiles";
  import { launcherState, progressInfo } from "../lib/stores/launcher";
  import ProfileCard from "../components/ProfileCard.svelte";
  import ProgressBar from "../components/ProgressBar.svelte";

  let isPlaying = $derived($launcherState !== "idle");

  const stateLabels: Record<string, string> = {
    idle: "Play",
    checking_java: "Checking Java...",
    downloading_java: "Downloading Java...",
    downloading_minecraft: "Downloading Minecraft...",
    installing_fabric: "Installing Fabric...",
    syncing_mods: "Syncing mods...",
    launching: "Launching...",
    running: "Running",
  };

  async function handlePlay() {
    if (isPlaying) return;
    try {
      await invoke("launch_game");
    } catch (e: any) {
      console.error("Launch failed:", e);
    }
  }

  async function handleSelectProfile(id: string) {
    if (isPlaying) return;
    await selectProfile(id);
  }
</script>

<div class="main-page">
  <h2>Select Profile</h2>

  <div class="profiles">
    {#each $profiles as profile}
      <ProfileCard
        {profile}
        selected={profile.id === $selectedProfileId}
        onselect={() => handleSelectProfile(profile.id)}
      />
    {/each}
    {#if $profiles.length === 0}
      <p class="no-profiles">No profiles configured. Add one in Settings.</p>
    {/if}
  </div>

  <div class="play-section">
    {#if $launcherState !== "idle" && $launcherState !== "running"}
      <div class="progress-area">
        <ProgressBar
          label={$progressInfo.label}
          detail={$progressInfo.detail}
          progress={$progressInfo.progress}
        />
      </div>
    {/if}

    <button
      class="play-btn"
      class:disabled={isPlaying}
      class:running={$launcherState === "running"}
      onclick={handlePlay}
    >
      {stateLabels[$launcherState] || "Play"}
    </button>
  </div>
</div>

<style>
  .main-page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  h2 {
    font-size: 16px;
    font-weight: 600;
    margin-bottom: 16px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 1px;
  }
  .profiles {
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex: 1;
    overflow-y: auto;
  }
  .no-profiles {
    color: var(--text-muted);
    font-size: 14px;
    padding: 24px;
    text-align: center;
  }
  .play-section {
    margin-top: 20px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }
  .progress-area {
    width: 100%;
    max-width: 400px;
  }
  .play-btn {
    background: var(--success);
    color: white;
    font-size: 18px;
    font-weight: 700;
    padding: 16px 64px;
    border-radius: var(--radius-lg);
    text-transform: uppercase;
    letter-spacing: 2px;
    transition: all var(--transition);
    box-shadow: 0 4px 20px rgba(0, 210, 160, 0.3);
  }
  .play-btn:hover:not(.disabled) {
    transform: translateY(-2px);
    box-shadow: 0 6px 24px rgba(0, 210, 160, 0.4);
  }
  .play-btn.disabled {
    background: var(--bg-tertiary);
    color: var(--text-muted);
    cursor: default;
    box-shadow: none;
  }
  .play-btn.running {
    background: var(--accent);
    box-shadow: 0 4px 20px var(--accent-glow);
  }
</style>
