<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { profiles, selectedProfileId, selectProfile, packStatuses, refreshPackStatuses } from "../lib/stores/profiles";
  import { launcherState, progressInfo } from "../lib/stores/launcher";
  import ProfileCard from "../components/ProfileCard.svelte";
  import ProgressBar from "../components/ProgressBar.svelte";

  let isPlaying = $derived($launcherState !== "idle");
  let selectedInstalled = $derived($packStatuses[$selectedProfileId] === "installed");

  // Config conflict dialog state
  let showConfigDialog = $state(false);
  let criticalConfigs = $state<string[]>([]);
  let criticalMessage = $state("");
  let optionalConflicts = $state<string[]>([]);

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

  async function handleInstall(profileId: string) {
    if (isPlaying) return;
    // Select the profile first, then run install (= prepare_and_sync without launch)
    await selectProfile(profileId);
    try {
      const result = await invoke<{
        status: string;
        critical?: string[];
        critical_message?: string;
        optional?: string[];
      }>("prepare_and_sync");

      if (result.status === "ConfigConflict") {
        criticalConfigs = result.critical ?? [];
        criticalMessage = result.critical_message ?? "";
        optionalConflicts = result.optional ?? [];
        if (optionalConflicts.length > 0 || criticalConfigs.length > 0) {
          showConfigDialog = true;
          // After config dialog, don't launch — just accept configs
          installOnly = true;
        } else {
          launcherState.set("idle");
        }
      } else {
        launcherState.set("idle");
      }
      await refreshPackStatuses();
    } catch (e: any) {
      console.error("Install failed:", e);
      launcherState.set("idle");
    }
  }

  let installOnly = $state(false);

  async function handleUninstall(profileId: string) {
    if (isPlaying) return;
    showConfirmUninstall = profileId;
  }

  let showConfirmUninstall = $state("");

  async function confirmUninstall() {
    const id = showConfirmUninstall;
    showConfirmUninstall = "";
    try {
      await invoke("uninstall_pack", { profileId: id });
      await refreshPackStatuses();
    } catch (e: any) {
      console.error("Uninstall failed:", e);
    }
  }

  async function handlePlay() {
    if (isPlaying || !selectedInstalled) return;
    try {
      const result = await invoke<{
        status: string;
        critical?: string[];
        critical_message?: string;
        optional?: string[];
      }>("prepare_and_sync");

      if (result.status === "ConfigConflict") {
        criticalConfigs = result.critical ?? [];
        criticalMessage = result.critical_message ?? "";
        optionalConflicts = result.optional ?? [];

        if (optionalConflicts.length > 0 || criticalConfigs.length > 0) {
          showConfigDialog = true;
        } else {
          await invoke("launch_after_sync");
        }
      } else {
        await invoke("launch_after_sync");
      }
    } catch (e: any) {
      console.error("Launch failed:", e);
      launcherState.set("idle");
    }
  }

  async function handleConfigChoice(keepUserConfigs: boolean) {
    showConfigDialog = false;
    criticalConfigs = [];
    criticalMessage = "";
    optionalConflicts = [];
    if (installOnly) {
      // Install-only mode: accept configs but don't launch
      installOnly = false;
      try {
        await invoke("resolve_configs", { keepUserConfigs });
      } catch (e: any) { console.error("resolve_configs:", e); }
      launcherState.set("idle");
      await refreshPackStatuses();
      return;
    }
    try {
      await invoke("resolve_configs_and_launch", { keepUserConfigs });
    } catch (e: any) {
      console.error("Launch failed after config choice:", e);
      launcherState.set("idle");
    }
  }

  /** Only critical configs, no optional — just acknowledge and launch. */
  async function handleCriticalAck() {
    showConfigDialog = false;
    criticalConfigs = [];
    criticalMessage = "";
    optionalConflicts = [];
    if (installOnly) {
      installOnly = false;
      try {
        await invoke("resolve_configs", { keepUserConfigs: false });
      } catch (e: any) { console.error("resolve_configs:", e); }
      launcherState.set("idle");
      await refreshPackStatuses();
      return;
    }
    try {
      await invoke("resolve_configs_and_launch", { keepUserConfigs: false });
    } catch (e: any) {
      console.error("Launch failed after critical ack:", e);
      launcherState.set("idle");
    }
  }

  async function handleSelectProfile(id: string) {
    if (isPlaying) return;
    await selectProfile(id);
  }

  // Repair state
  let isRepairing = $state(false);
  let showRepairResult = $state(false);
  let repairResult = $state<{
    removed_mods: string[];
    removed_datapacks: string[];
    restored_configs: string[];
    resynced: boolean;
  } | null>(null);
  let repairError = $state("");

  async function handleRepair() {
    if (isPlaying || isRepairing) return;
    isRepairing = true;
    repairError = "";
    try {
      const result = await invoke<{
        removed_mods: string[];
        removed_datapacks: string[];
        restored_configs: string[];
        resynced: boolean;
      }>("repair_pack");
      repairResult = result;
      showRepairResult = true;
    } catch (e: any) {
      repairError = String(e);
      showRepairResult = true;
    } finally {
      isRepairing = false;
      launcherState.set("idle");
    }
  }

  async function openInstanceFolder() {
    try {
      await invoke("open_instance_dir");
    } catch (e: any) {
      console.error("Failed to open folder:", e);
    }
  }
</script>

<div class="main-page">
  <h2>Select Profile</h2>

  <div class="profiles">
    {#each $profiles as profile}
      <ProfileCard
        {profile}
        selected={profile.id === $selectedProfileId}
        installed={$packStatuses[profile.id] === "installed"}
        onselect={() => handleSelectProfile(profile.id)}
        oninstall={() => handleInstall(profile.id)}
        onuninstall={() => handleUninstall(profile.id)}
      />
    {/each}
    {#if $profiles.length === 0}
      <p class="no-profiles">No profiles configured. Add one in Settings.</p>
    {/if}
  </div>

  <div class="play-section">
    {#if $launcherState !== "idle" && $launcherState !== "running" && !showConfigDialog}
      <div class="progress-area">
        <ProgressBar
          label={$progressInfo.label}
          detail={$progressInfo.detail}
          progress={$progressInfo.progress}
        />
      </div>
    {/if}

    <div class="btn-row">
      <button
        class="play-btn"
        class:disabled={isPlaying || !selectedInstalled}
        class:running={$launcherState === "running"}
        onclick={handlePlay}
      >
        {#if !selectedInstalled && $launcherState === "idle"}
          Not installed
        {:else}
          {stateLabels[$launcherState] || "Play"}
        {/if}
      </button>
      <button
        class="folder-btn"
        class:disabled={isRepairing}
        onclick={handleRepair}
        title="Verify & repair pack"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
        </svg>
      </button>
      <button class="folder-btn" onclick={openInstanceFolder} title="Open instance folder">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
      </button>
    </div>
  </div>
</div>

{#if showConfigDialog}
  <div class="dialog-overlay" role="dialog" aria-modal="true">
    <div class="dialog">
      <h3>Configuration update</h3>

      {#if criticalConfigs.length > 0}
        <div class="critical-section">
          <p class="critical-label">Required updates applied</p>
          <p class="critical-msg">{criticalMessage || "Some configs were force-updated for compatibility."}</p>
          <div class="conflict-list critical">
            {#each criticalConfigs.slice(0, 5) as file}
              <div class="conflict-file">{file}</div>
            {/each}
            {#if criticalConfigs.length > 5}
              <div class="conflict-more">+{criticalConfigs.length - 5} more...</div>
            {/if}
          </div>
        </div>
      {/if}

      {#if optionalConflicts.length > 0}
        <p>
          {optionalConflicts.length} config file{optionalConflicts.length > 1 ? 's' : ''} you
          customized {optionalConflicts.length > 1 ? 'have' : 'has'} also been updated.
        </p>
        <div class="conflict-list">
          {#each optionalConflicts.slice(0, 8) as file}
            <div class="conflict-file">{file}</div>
          {/each}
          {#if optionalConflicts.length > 8}
            <div class="conflict-more">+{optionalConflicts.length - 8} more...</div>
          {/if}
        </div>
        <div class="dialog-buttons">
          <button class="btn-secondary" onclick={() => handleConfigChoice(true)}>
            Keep my settings
          </button>
          <button class="btn-primary" onclick={() => handleConfigChoice(false)}>
            Use pack defaults
          </button>
        </div>
      {:else}
        <div class="dialog-buttons">
          <button class="btn-primary" onclick={handleCriticalAck}>
            OK
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

{#if showConfirmUninstall}
  <div class="dialog-overlay" role="dialog" aria-modal="true">
    <div class="dialog">
      <h3>Uninstall pack</h3>
      <p>This will delete all downloaded files for this profile. You can reinstall it later.</p>
      <div class="dialog-buttons">
        <button class="btn-secondary" onclick={() => { showConfirmUninstall = ""; }}>Cancel</button>
        <button class="btn-danger" onclick={confirmUninstall}>Uninstall</button>
      </div>
    </div>
  </div>
{/if}

{#if showRepairResult}
  <div class="dialog-overlay" role="dialog" aria-modal="true">
    <div class="dialog">
      {#if repairError}
        <h3>Repair failed</h3>
        <p class="repair-error">{repairError}</p>
      {:else if repairResult && (repairResult.removed_mods.length > 0 || repairResult.removed_datapacks.length > 0 || repairResult.restored_configs.length > 0)}
        <h3>Pack repaired</h3>
        {#if repairResult.removed_mods.length > 0}
          <p class="repair-category">Removed {repairResult.removed_mods.length} unauthorized mod{repairResult.removed_mods.length > 1 ? 's' : ''}</p>
          <div class="conflict-list">
            {#each repairResult.removed_mods as name}
              <div class="conflict-file">{name}</div>
            {/each}
          </div>
        {/if}
        {#if repairResult.removed_datapacks.length > 0}
          <p class="repair-category">Removed {repairResult.removed_datapacks.length} unauthorized datapack{repairResult.removed_datapacks.length > 1 ? 's' : ''}</p>
          <div class="conflict-list">
            {#each repairResult.removed_datapacks as name}
              <div class="conflict-file">{name}</div>
            {/each}
          </div>
        {/if}
        {#if repairResult.restored_configs.length > 0}
          <p class="repair-category">Restored {repairResult.restored_configs.length} config{repairResult.restored_configs.length > 1 ? 's' : ''}</p>
          <div class="conflict-list">
            {#each repairResult.restored_configs.slice(0, 10) as name}
              <div class="conflict-file">{name}</div>
            {/each}
            {#if repairResult.restored_configs.length > 10}
              <div class="conflict-more">+{repairResult.restored_configs.length - 10} more...</div>
            {/if}
          </div>
        {/if}
      {:else}
        <h3>Pack is clean</h3>
        <p>No issues found. All mods, datapacks, and configs match the expected pack.</p>
      {/if}
      <div class="dialog-buttons">
        <button class="btn-primary" onclick={() => { showRepairResult = false; repairResult = null; repairError = ""; }}>
          OK
        </button>
      </div>
    </div>
  </div>
{/if}

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
  .btn-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .folder-btn {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    padding: 16px;
    border-radius: var(--radius-lg);
    transition: all var(--transition);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .folder-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  .dialog {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 28px;
    max-width: 460px;
    width: 90%;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  }
  .dialog h3 {
    font-size: 18px;
    font-weight: 600;
    margin-bottom: 12px;
    color: var(--text-primary);
  }
  .dialog p {
    font-size: 14px;
    color: var(--text-secondary);
    margin-bottom: 16px;
    line-height: 1.5;
  }
  .conflict-list {
    background: var(--bg-primary);
    border-radius: var(--radius);
    padding: 12px;
    margin-bottom: 20px;
    max-height: 200px;
    overflow-y: auto;
  }
  .conflict-file {
    font-family: monospace;
    font-size: 12px;
    color: var(--text-muted);
    padding: 3px 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .conflict-more {
    font-size: 12px;
    color: var(--accent);
    padding-top: 6px;
  }
  .critical-section {
    margin-bottom: 16px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border);
  }
  .critical-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--warning, #f0a030);
    margin-bottom: 4px;
  }
  .critical-msg {
    font-size: 13px;
    color: var(--text-muted);
    margin-bottom: 10px;
    line-height: 1.4;
  }
  .conflict-list.critical {
    border-left: 3px solid var(--warning, #f0a030);
  }
  .repair-category {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 6px;
    margin-top: 12px;
  }
  .repair-category:first-of-type {
    margin-top: 0;
  }
  .repair-error {
    color: var(--error, #e05050);
    font-family: monospace;
    font-size: 13px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .dialog-buttons {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
  }
  .btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    padding: 10px 20px;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 500;
    transition: all var(--transition);
  }
  .btn-secondary:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .btn-primary {
    background: var(--accent);
    color: white;
    padding: 10px 20px;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 600;
    transition: all var(--transition);
  }
  .btn-primary:hover {
    filter: brightness(1.1);
  }
  .btn-danger {
    background: var(--error, #e05050);
    color: white;
    padding: 10px 20px;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 600;
    transition: all var(--transition);
  }
  .btn-danger:hover {
    filter: brightness(1.1);
  }
</style>
