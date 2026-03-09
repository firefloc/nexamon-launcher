<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { profiles, selectedProfileId, selectProfile, packStatuses, refreshPackStatuses } from "../lib/stores/profiles";
  import { launcherState, progressInfo } from "../lib/stores/launcher";
  import { settings } from "../lib/stores/settings";
  import ProfileCard from "../components/ProfileCard.svelte";
  import ProgressBar from "../components/ProgressBar.svelte";
  import Dialog from "../components/Dialog.svelte";
  import { t } from "../lib/i18n";
  import { debugOutlines } from "../lib/stores/dev";

  let isPlaying = $derived($launcherState !== "idle");
  let selectedInstalled = $derived($packStatuses[$selectedProfileId] === "installed");

  // Config conflict dialog
  let showConfigDialog = $state(false);
  let criticalConfigs = $state<string[]>([]);
  let criticalMessage = $state("");
  let optionalConflicts = $state<string[]>([]);
  let installOnly = $state(false);

  // Install error dialog
  let installError = $state("");

  // Confirm uninstall dialog
  let showConfirmUninstall = $state("");

  let selectedProfile = $derived($profiles.find(p => p.id === $selectedProfileId) ?? null);
  let showProgress = $derived($launcherState !== "idle" && $launcherState !== "running" && !showConfigDialog);

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
        if ($settings.auto_accept_configs) {
          handleConfigChoice(false);
          return true;
        }
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
    } catch (e: any) {
      console.error("Install failed:", e);
      installError = String(e);
      launcherState.set("idle");
    } finally {
      await refreshPackStatuses();
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
      installError = String(e);
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

  function handleCancel() {
    invoke("cancel_operation");
    launcherState.set("idle");
  }
</script>

<div class="main-page" class:debug={$debugOutlines}>
  <!-- Hero content area -->
  <div class="hero-div">
    <p class="hero-placeholder">{$t("main.hero_wip")}</p>
  </div>

  <!-- Hero row: title left, play right -->
  <div class="hero-row">
    <div class="selected-info">
      {#if selectedProfile}
        <h2 class="profile-title">{selectedProfile.name}</h2>
        <p class="profile-desc">{selectedProfile.description}</p>
      {:else}
        <p class="no-selection">{$t("main.no_profiles")}</p>
      {/if}
    </div>
    <button
      class="play-btn"
      class:disabled={isPlaying || !selectedInstalled}
      class:running={$launcherState === "running"}
      onclick={handlePlay}
    >
      {#if !selectedInstalled && $launcherState === "idle"}
        {$t("main.not_installed")}
      {:else if $launcherState === "idle"}
        {$t("main.play")}
      {:else}
        {$t("main." + $launcherState)}
      {/if}
    </button>
  </div>

  <!-- Progress bar (above profile buttons, full width, always reserves space) -->
  <div class="progress-row" class:hidden={!showProgress}>
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

  <!-- Profile selector -->
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
  </div>
</div>

<!-- Config conflict dialog -->
{#if showConfigDialog}
  <Dialog title={$t("dialog.config_update")}>
    {#snippet children()}
      {#if criticalConfigs.length > 0}
        <div class="critical-section">
          <p class="label-warning">{$t("dialog.required_updates")}</p>
          <p class="text-muted">{criticalMessage || $t("dialog.force_updated")}</p>
          <div class="file-list warning">
            {#each criticalConfigs.slice(0, 5) as file}
              <div class="file-entry">{file}</div>
            {/each}
            {#if criticalConfigs.length > 5}
              <div class="file-more">{$t("dialog.more", { n: criticalConfigs.length - 5 })}</div>
            {/if}
          </div>
        </div>
      {/if}
      {#if optionalConflicts.length > 0}
        <p class="text-secondary">
          {optionalConflicts.length > 1 ? $t("dialog.optional_plural", { n: optionalConflicts.length }) : $t("dialog.optional_single")}
        </p>
        <div class="file-list">
          {#each optionalConflicts.slice(0, 8) as file}
            <div class="file-entry">{file}</div>
          {/each}
          {#if optionalConflicts.length > 8}
            <div class="file-more">{$t("dialog.more", { n: optionalConflicts.length - 8 })}</div>
          {/if}
        </div>
      {/if}
    {/snippet}
    {#snippet actions()}
      {#if optionalConflicts.length > 0}
        <button class="btn btn-secondary" onclick={() => handleConfigChoice(true)}>{$t("dialog.keep_settings")}</button>
        <button class="btn btn-primary" onclick={() => handleConfigChoice(false)}>{$t("dialog.use_defaults")}</button>
      {:else}
        <button class="btn btn-primary" onclick={() => handleConfigChoice(false)}>{$t("dialog.ok")}</button>
      {/if}
    {/snippet}
  </Dialog>
{/if}

<!-- Install/launch error -->
{#if installError}
  <Dialog title={$t("dialog.error")}>
    {#snippet children()}
      <p class="text-error mono">{installError}</p>
    {/snippet}
    {#snippet actions()}
      <button class="btn btn-primary" onclick={() => { installError = ""; }}>OK</button>
    {/snippet}
  </Dialog>
{/if}

<!-- Uninstall confirmation -->
{#if showConfirmUninstall}
  <Dialog title={$t("dialog.uninstall")}>
    {#snippet children()}
      <p class="text-secondary">{$t("dialog.uninstall_confirm")}</p>
    {/snippet}
    {#snippet actions()}
      <button class="btn btn-secondary" onclick={() => { showConfirmUninstall = ""; }}>{$t("dialog.cancel")}</button>
      <button class="btn btn-danger" onclick={confirmUninstall}>{$t("dialog.uninstall_btn")}</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .main-page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  /* Hero content area */
  .hero-div {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 0;
  }
  .hero-placeholder {
    color: var(--text-muted);
    font-size: 14px;
    text-align: center;
    max-width: 300px;
    line-height: 1.5;
  }

  /* Hero row: title left, play right */
  .hero-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 16px 0;
  }
  .selected-info {
    text-align: left;
    min-width: 0;
  }
  .profile-title {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: 4px;
  }
  .profile-desc {
    font-size: 13px;
    color: var(--text-secondary);
  }
  .no-selection {
    color: var(--text-muted);
    font-size: 14px;
  }

  /* Profile selector (bottom) */
  .profiles {
    display: flex;
    flex-direction: row;
    gap: 8px;
    padding-top: 16px;
    border-top: 1px solid var(--border-subtle);
  }
  .progress-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
  }
  .progress-row.hidden { visibility: hidden; }
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

  /* Debug outlines (toggled from Dev page) */
  .main-page.debug { outline: 2px dashed red; }
  .main-page.debug .hero-row { outline: 2px dashed lime; }
  .main-page.debug .selected-info { outline: 2px dashed cyan; }
  .main-page.debug .play-btn { outline: 2px dashed orange; }
  .main-page.debug .hero-div { outline: 2px dashed yellow; }
  .main-page.debug .profiles { outline: 2px dashed magenta; }
</style>
