<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { settings, saveSettings, type Settings } from "../lib/stores/settings";
  import { launcherState } from "../lib/stores/launcher";
  import Dialog from "../components/Dialog.svelte";
  import { t } from "../lib/i18n";

  let local = $state<Settings>({ ...$settings });

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

  async function handleSave() {
    await saveSettings(local);
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

<div class="maintenance-page">
  <h2>{$t("nav.maintenance")}</h2>

  <div class="section">
    <h3>{$t("maintenance.configs")}</h3>
    <label class="field checkbox">
      <input type="checkbox" bind:checked={local.auto_accept_configs} onchange={handleSave} />
      <span>{$t("settings.auto_accept_configs")}</span>
    </label>
  </div>

  <div class="section">
    <h3>{$t("maintenance.tools")}</h3>
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
  .maintenance-page {
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
