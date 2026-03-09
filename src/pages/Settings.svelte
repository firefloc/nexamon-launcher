<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { settings, saveSettings, type Settings } from "../lib/stores/settings";
  import { launcherState } from "../lib/stores/launcher";
  import { t, locale, type Locale } from "../lib/i18n";
  import Dialog from "../components/Dialog.svelte";

  let local = $state<Settings>({ ...$settings });
  let saved = $state(false);
  let maxRamMb = $state(16384);

  // Repair
  let isRepairing = $state(false);
  let showRepairResult = $state(false);
  let repairResult = $state<{
    removed_mods: string[];
    removed_datapacks: string[];
    restored_configs: string[];
    resynced: boolean;
  } | null>(null);
  let repairError = $state("");

  $effect(() => {
    local = { ...$settings };
  });

  $effect(() => {
    invoke<number>("get_system_ram_mb").then((ram) => {
      maxRamMb = Math.floor(ram / 512) * 512;
    });
  });

  async function handleSave() {
    await saveSettings(local);
    saved = true;
    setTimeout(() => (saved = false), 2000);
  }

  async function handleRepair() {
    if (isRepairing) return;
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
</script>

<div class="settings-page">
  <h2>{$t("settings.title")}</h2>

  <div class="section">
    <h3>{$t("settings.performance")}</h3>
    <label class="field">
      <span>{$t("settings.ram")}</span>
      <div class="ram-control">
        <input
          type="range"
          min="2048"
          max={maxRamMb}
          step="512"
          bind:value={local.ram_mb}
        />
        <span class="ram-value">{(local.ram_mb / 1024).toFixed(1)} GB</span>
      </div>
    </label>
  </div>

  <div class="section">
    <h3>{$t("settings.java")}</h3>
    <label class="field">
      <span>{$t("settings.java_path")}</span>
      <input
        type="text"
        placeholder={$t("settings.auto_detect")}
        bind:value={local.java_path}
      />
    </label>
  </div>

  <div class="section">
    <h3>{$t("settings.launcher")}</h3>
    <label class="field checkbox">
      <input type="checkbox" bind:checked={local.close_on_launch} />
      <span>{$t("settings.close_on_launch")}</span>
    </label>
    <label class="field checkbox">
      <input type="checkbox" bind:checked={local.auto_accept_configs} />
      <span>{$t("settings.auto_accept_configs")}</span>
    </label>
  </div>

  <div class="section">
    <h3>{$t("settings.language")}</h3>
    <label class="field">
      <select class="lang-select" onchange={(e) => locale.set((e.currentTarget as HTMLSelectElement).value as Locale)}>
        <option value="fr" selected={$locale === "fr"}>Francais</option>
        <option value="en" selected={$locale === "en"}>English</option>
      </select>
    </label>
  </div>

  <div class="section">
    <h3>{$t("settings.maintenance")}</h3>
    <div class="tool-row">
      <div class="tool-info">
        <span class="tool-name">{$t("settings.repair")}</span>
        <span class="tool-desc">{$t("settings.repair_desc")}</span>
      </div>
      <button class="tool-btn" class:disabled={isRepairing} onclick={handleRepair}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
        </svg>
        {isRepairing ? $t("settings.repairing") : $t("settings.repair")}
      </button>
    </div>
    <div class="tool-row">
      <div class="tool-info">
        <span class="tool-name">{$t("settings.open_folder")}</span>
        <span class="tool-desc">{$t("settings.open_folder_desc")}</span>
      </div>
      <button class="tool-btn" onclick={openInstanceFolder}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        {$t("settings.open_folder")}
      </button>
    </div>
  </div>

  <div class="actions">
    <button class="save-btn" onclick={handleSave}>
      {saved ? $t("settings.saved") + " ✓" : $t("settings.save")}
    </button>
  </div>
</div>

<!-- Repair result dialog -->
{#if showRepairResult}
  {@const hasChanges = repairResult && (repairResult.removed_mods.length > 0 || repairResult.removed_datapacks.length > 0 || repairResult.restored_configs.length > 0)}
  <Dialog title={repairError ? $t("dialog.repair_failed") : hasChanges ? $t("dialog.pack_repaired") : $t("dialog.pack_clean")}>
    {#snippet children()}
      {#if repairError}
        <p class="text-error mono">{repairError}</p>
      {:else if hasChanges && repairResult}
        {#if repairResult.removed_mods.length > 0}
          <p class="label-muted">{$t("dialog.removed_mods", { n: repairResult.removed_mods.length })}</p>
          <div class="file-list">
            {#each repairResult.removed_mods as name}<div class="file-entry">{name}</div>{/each}
          </div>
        {/if}
        {#if repairResult.removed_datapacks.length > 0}
          <p class="label-muted">{$t("dialog.removed_datapacks", { n: repairResult.removed_datapacks.length })}</p>
          <div class="file-list">
            {#each repairResult.removed_datapacks as name}<div class="file-entry">{name}</div>{/each}
          </div>
        {/if}
        {#if repairResult.restored_configs.length > 0}
          <p class="label-muted">{$t("dialog.restored_configs", { n: repairResult.restored_configs.length })}</p>
          <div class="file-list">
            {#each repairResult.restored_configs.slice(0, 10) as name}<div class="file-entry">{name}</div>{/each}
            {#if repairResult.restored_configs.length > 10}
              <div class="file-more">{$t("dialog.more", { n: repairResult.restored_configs.length - 10 })}</div>
            {/if}
          </div>
        {/if}
      {:else}
        <p class="text-secondary">{$t("dialog.no_issues")}</p>
      {/if}
    {/snippet}
    {#snippet actions()}
      <button class="btn btn-primary" onclick={() => { showRepairResult = false; repairResult = null; repairError = ""; }}>{$t("dialog.ok")}</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .settings-page {
    max-width: 600px;
  }
  h2 {
    font-size: 16px;
    font-weight: 600;
    margin-bottom: 24px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 1px;
  }
  .section {
    margin-bottom: 24px;
  }
  h3 {
    font-size: 13px;
    color: var(--text-muted);
    margin-bottom: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 12px;
  }
  .field > span {
    font-size: 13px;
    color: var(--text-secondary);
  }
  .field.checkbox {
    flex-direction: row;
    align-items: center;
    gap: 8px;
  }
  .field.checkbox input {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }
  .ram-control {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .ram-control input[type="range"] {
    flex: 1;
    accent-color: var(--accent);
    background: transparent;
    border: none;
    padding: 0;
  }
  .ram-value {
    font-weight: 600;
    font-size: 14px;
    min-width: 60px;
    text-align: right;
  }
  .actions {
    margin-top: 24px;
  }
  .save-btn {
    background: var(--accent);
    color: white;
    padding: 10px 24px;
    border-radius: var(--radius);
    font-weight: 600;
    font-size: 14px;
    transition: background var(--transition);
  }
  .save-btn:hover {
    background: var(--accent-hover);
  }
  .lang-select {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 8px 12px;
    font-size: 14px;
    cursor: pointer;
  }
  .lang-select:focus {
    border-color: var(--accent);
    outline: none;
  }
  .tool-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 12px;
  }
  .tool-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .tool-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }
  .tool-desc {
    font-size: 12px;
    color: var(--text-muted);
  }
  .tool-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    font-size: 13px;
    cursor: pointer;
    transition: all var(--transition);
    white-space: nowrap;
    flex-shrink: 0;
  }
  .tool-btn:hover:not(.disabled) {
    background: var(--bg-card);
    color: var(--text-primary);
    border-color: var(--accent);
  }
  .tool-btn.disabled {
    opacity: 0.5;
    cursor: default;
  }

  /* Dialog styles */
  .text-secondary { font-size: 14px; color: var(--text-secondary); margin-bottom: 16px; line-height: 1.5; }
  .text-error { color: var(--danger); line-height: 1.5; white-space: pre-wrap; word-break: break-word; }
  .mono { font-family: monospace; font-size: 13px; }
  .label-muted { font-size: 13px; font-weight: 600; color: var(--text-secondary); margin-bottom: 6px; margin-top: 12px; }
  .label-muted:first-of-type { margin-top: 0; }
  .file-list {
    background: var(--bg-primary);
    border-radius: var(--radius);
    padding: 12px;
    margin-bottom: 12px;
    max-height: 200px;
    overflow-y: auto;
  }
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
</style>
