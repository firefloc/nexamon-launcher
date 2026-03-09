<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { profiles, selectedProfileId, selectProfile, packStatuses, refreshPackStatuses } from "../lib/stores/profiles";
  import { launcherState, progressInfo } from "../lib/stores/launcher";
  import ProfileCard from "../components/ProfileCard.svelte";
  import ProgressBar from "../components/ProgressBar.svelte";
  import Dialog from "../components/Dialog.svelte";

  let isPlaying = $derived($launcherState !== "idle");
  let selectedInstalled = $derived($packStatuses[$selectedProfileId] === "installed");

  // Config conflict dialog
  let showConfigDialog = $state(false);
  let criticalConfigs = $state<string[]>([]);
  let criticalMessage = $state("");
  let optionalConflicts = $state<string[]>([]);
  let installOnly = $state(false);

  // Confirm uninstall dialog
  let showConfirmUninstall = $state("");

  // Repair dialog
  let isRepairing = $state(false);
  let showRepairResult = $state(false);
  let repairResult = $state<{
    removed_mods: string[];
    removed_datapacks: string[];
    restored_configs: string[];
    resynced: boolean;
  } | null>(null);
  let repairError = $state("");

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

  type SyncResult = {
    status: string;
    critical?: string[];
    critical_message?: string;
    optional?: string[];
  };

  function handleSyncResult(result: SyncResult, isInstallOnly: boolean): boolean {
    if (result.status === "ConfigConflict") {
      criticalConfigs = result.critical ?? [];
      criticalMessage = result.critical_message ?? "";
      optionalConflicts = result.optional ?? [];
      if (criticalConfigs.length > 0 || optionalConflicts.length > 0) {
        showConfigDialog = true;
        installOnly = isInstallOnly;
        return true;
      }
    }
    return false;
  }

  function resetConfigDialog() {
    showConfigDialog = false;
    criticalConfigs = [];
    criticalMessage = "";
    optionalConflicts = [];
  }

  async function handleInstall(profileId: string) {
    if (isPlaying) return;
    await selectProfile(profileId);
    try {
      const result = await invoke<SyncResult>("prepare_and_sync");
      if (!handleSyncResult(result, true)) {
        launcherState.set("idle");
      }
      await refreshPackStatuses();
    } catch (e: any) {
      console.error("Install failed:", e);
      launcherState.set("idle");
    }
  }

  async function handlePlay() {
    if (isPlaying || !selectedInstalled) return;
    try {
      const result = await invoke<SyncResult>("prepare_and_sync");
      if (!handleSyncResult(result, false)) {
        await invoke("launch_after_sync");
      }
    } catch (e: any) {
      console.error("Launch failed:", e);
      launcherState.set("idle");
    }
  }

  async function handleConfigChoice(keepUserConfigs: boolean) {
    resetConfigDialog();
    if (installOnly) {
      installOnly = false;
      try { await invoke("resolve_configs", { keepUserConfigs }); } catch (e: any) { console.error(e); }
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

  async function handleRepair() {
    if (isPlaying || isRepairing) return;
    isRepairing = true;
    repairError = "";
    try {
      repairResult = await invoke("repair_pack");
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
    try { await invoke("open_instance_dir"); } catch (e: any) { console.error(e); }
  }

  function handleCancel() {
    invoke("cancel_operation");
    launcherState.set("idle");
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
        onselect={() => { if (!isPlaying) selectProfile(profile.id); }}
        oninstall={() => handleInstall(profile.id)}
        onuninstall={() => { showConfirmUninstall = profile.id; }}
      />
    {/each}
    {#if $profiles.length === 0}
      <p class="no-profiles">No profiles configured. Add one in Settings.</p>
    {/if}
  </div>

  <div class="play-section">
    {#if $launcherState !== "idle" && $launcherState !== "running" && !showConfigDialog}
      <div class="progress-row">
        <div class="progress-area">
          <ProgressBar
            label={$progressInfo.label}
            detail={$progressInfo.detail}
            progress={$progressInfo.progress}
          />
        </div>
        <button class="cancel-btn" onclick={handleCancel} title="Cancel">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
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
      <button class="tool-btn" class:disabled={isRepairing} onclick={handleRepair} title="Verify & repair">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
        </svg>
      </button>
      <button class="tool-btn" onclick={openInstanceFolder} title="Open folder">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
      </button>
    </div>
  </div>
</div>

<!-- Config conflict dialog -->
{#if showConfigDialog}
  <Dialog title="Configuration update">
    {#snippet children()}
      {#if criticalConfigs.length > 0}
        <div class="critical-section">
          <p class="label-warning">Required updates applied</p>
          <p class="text-muted">{criticalMessage || "Some configs were force-updated for compatibility."}</p>
          <div class="file-list warning">
            {#each criticalConfigs.slice(0, 5) as file}
              <div class="file-entry">{file}</div>
            {/each}
            {#if criticalConfigs.length > 5}
              <div class="file-more">+{criticalConfigs.length - 5} more...</div>
            {/if}
          </div>
        </div>
      {/if}
      {#if optionalConflicts.length > 0}
        <p class="text-secondary">
          {optionalConflicts.length} config file{optionalConflicts.length > 1 ? 's' : ''} you
          customized {optionalConflicts.length > 1 ? 'have' : 'has'} also been updated.
        </p>
        <div class="file-list">
          {#each optionalConflicts.slice(0, 8) as file}
            <div class="file-entry">{file}</div>
          {/each}
          {#if optionalConflicts.length > 8}
            <div class="file-more">+{optionalConflicts.length - 8} more...</div>
          {/if}
        </div>
      {/if}
    {/snippet}
    {#snippet actions()}
      {#if optionalConflicts.length > 0}
        <button class="btn btn-secondary" onclick={() => handleConfigChoice(true)}>Keep my settings</button>
        <button class="btn btn-primary" onclick={() => handleConfigChoice(false)}>Use pack defaults</button>
      {:else}
        <button class="btn btn-primary" onclick={() => handleConfigChoice(false)}>OK</button>
      {/if}
    {/snippet}
  </Dialog>
{/if}

<!-- Uninstall confirmation -->
{#if showConfirmUninstall}
  <Dialog title="Uninstall pack">
    {#snippet children()}
      <p class="text-secondary">This will delete all downloaded files for this profile. You can reinstall it later.</p>
    {/snippet}
    {#snippet actions()}
      <button class="btn btn-secondary" onclick={() => { showConfirmUninstall = ""; }}>Cancel</button>
      <button class="btn btn-danger" onclick={confirmUninstall}>Uninstall</button>
    {/snippet}
  </Dialog>
{/if}

<!-- Repair result -->
{#if showRepairResult}
  {@const hasChanges = repairResult && (repairResult.removed_mods.length > 0 || repairResult.removed_datapacks.length > 0 || repairResult.restored_configs.length > 0)}
  <Dialog title={repairError ? "Repair failed" : hasChanges ? "Pack repaired" : "Pack is clean"}>
    {#snippet children()}
      {#if repairError}
        <p class="text-error mono">{repairError}</p>
      {:else if hasChanges && repairResult}
        {#if repairResult.removed_mods.length > 0}
          <p class="label-muted">Removed {repairResult.removed_mods.length} unauthorized mod{repairResult.removed_mods.length > 1 ? 's' : ''}</p>
          <div class="file-list">
            {#each repairResult.removed_mods as name}<div class="file-entry">{name}</div>{/each}
          </div>
        {/if}
        {#if repairResult.removed_datapacks.length > 0}
          <p class="label-muted">Removed {repairResult.removed_datapacks.length} unauthorized datapack{repairResult.removed_datapacks.length > 1 ? 's' : ''}</p>
          <div class="file-list">
            {#each repairResult.removed_datapacks as name}<div class="file-entry">{name}</div>{/each}
          </div>
        {/if}
        {#if repairResult.restored_configs.length > 0}
          <p class="label-muted">Restored {repairResult.restored_configs.length} config{repairResult.restored_configs.length > 1 ? 's' : ''}</p>
          <div class="file-list">
            {#each repairResult.restored_configs.slice(0, 10) as name}<div class="file-entry">{name}</div>{/each}
            {#if repairResult.restored_configs.length > 10}
              <div class="file-more">+{repairResult.restored_configs.length - 10} more...</div>
            {/if}
          </div>
        {/if}
      {:else}
        <p class="text-secondary">No issues found. All mods, datapacks, and configs match the expected pack.</p>
      {/if}
    {/snippet}
    {#snippet actions()}
      <button class="btn btn-primary" onclick={() => { showRepairResult = false; repairResult = null; repairError = ""; }}>OK</button>
    {/snippet}
  </Dialog>
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

  /* Play section */
  .play-section {
    margin-top: 20px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }
  .progress-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    max-width: 440px;
  }
  .progress-area { flex: 1; min-width: 0; }
  .cancel-btn {
    background: var(--bg-tertiary);
    color: var(--text-muted);
    border: none;
    padding: 8px;
    border-radius: var(--radius);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition);
    flex-shrink: 0;
  }
  .cancel-btn:hover { background: var(--danger); color: white; }
  .btn-row {
    display: flex;
    align-items: center;
    gap: 12px;
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
  .tool-btn {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    padding: 16px;
    border-radius: var(--radius-lg);
    transition: all var(--transition);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .tool-btn:hover:not(.disabled) {
    background: var(--bg-card);
    color: var(--text-primary);
  }
  .tool-btn.disabled { opacity: 0.5; cursor: default; }

  /* Dialog content */
  .text-secondary { font-size: 14px; color: var(--text-secondary); margin-bottom: 16px; line-height: 1.5; }
  .text-muted { font-size: 13px; color: var(--text-muted); margin-bottom: 10px; line-height: 1.4; }
  .text-error { color: var(--danger); line-height: 1.5; white-space: pre-wrap; word-break: break-word; }
  .mono { font-family: monospace; font-size: 13px; }
  .label-warning { font-size: 13px; font-weight: 600; color: var(--warning); margin-bottom: 4px; }
  .label-muted { font-size: 13px; font-weight: 600; color: var(--text-secondary); margin-bottom: 6px; margin-top: 12px; }
  .label-muted:first-of-type { margin-top: 0; }
  .critical-section { margin-bottom: 16px; padding-bottom: 16px; border-bottom: 1px solid var(--border); }
  .file-list {
    background: var(--bg-primary);
    border-radius: var(--radius);
    padding: 12px;
    margin-bottom: 12px;
    max-height: 200px;
    overflow-y: auto;
  }
  .file-list.warning { border-left: 3px solid var(--warning); }
  .file-entry {
    font-family: monospace;
    font-size: 12px;
    color: var(--text-muted);
    padding: 3px 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .file-more { font-size: 12px; color: var(--accent); padding-top: 6px; }

  /* Buttons */
  .btn {
    padding: 10px 20px;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 600;
    transition: all var(--transition);
    border: none;
    cursor: pointer;
  }
  .btn-primary { background: var(--accent); color: white; }
  .btn-primary:hover { filter: brightness(1.1); }
  .btn-secondary { background: var(--bg-tertiary); color: var(--text-secondary); font-weight: 500; }
  .btn-secondary:hover { background: var(--bg-card); color: var(--text-primary); }
  .btn-danger { background: var(--danger); color: white; }
  .btn-danger:hover { filter: brightness(1.1); }
</style>
