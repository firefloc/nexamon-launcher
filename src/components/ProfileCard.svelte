<script lang="ts">
  import type { Profile } from "../lib/stores/profiles";
  import { t } from "../lib/i18n";

  let { profile, selected = false, installed = false, systemRamMb = 16384, onselect, oninstall, onuninstall }: {
    profile: Profile;
    selected: boolean;
    installed: boolean;
    systemRamMb?: number;
    onselect: () => void;
    oninstall: () => void;
    onuninstall: () => void;
  } = $props();

  let ramInsufficient = $derived(profile.recommended_ram_mb > 0 && systemRamMb < profile.recommended_ram_mb);
  let ramLabel = $derived(profile.recommended_ram_mb > 0 ? `${(profile.recommended_ram_mb / 1024).toFixed(0)} GB RAM` : "");
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="card" class:selected role="button" tabindex="0" onclick={onselect} onkeydown={(e) => { if (e.key === 'Enter') onselect(); }}>
  <div class="card-icon">
    {#if profile.icon === "low"}
      <svg viewBox="0 0 24 24" fill="none" stroke="var(--success)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M17 8C17 10.761 14.761 13 12 13C9.239 13 7 10.761 7 8"/><path d="M12 2v11"/><path d="M8 16c-3 1-5 3-5 5h18c0-2-2-4-5-5"/><path d="M12 13c2 2 4 5 4 7"/>
      </svg>
    {:else if profile.icon === "high"}
      <svg viewBox="0 0 24 24" fill="none" stroke="var(--accent)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/>
      </svg>
    {:else if profile.icon === "ultra"}
      <svg viewBox="0 0 24 24" fill="none" stroke="var(--warning)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
      </svg>
    {:else}
      <svg viewBox="0 0 24 24" fill="none" stroke="var(--text-secondary)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
      </svg>
    {/if}
  </div>
  <div class="card-body">
    <div class="card-top">
      <span class="card-name">{profile.name}</span>
      {#if ramLabel}
        <span class="ram-hint" class:insufficient={ramInsufficient} title={ramInsufficient ? $t("profile.ram_insufficient") : $t("profile.ram_recommended")}>
          {ramLabel}
        </span>
      {/if}
      {#if installed}
        <span class="status">{$t("profile.installed")}</span>
      {/if}
    </div>
    {#if installed}
      <button
        class="action-btn uninstall"
        onclick={(e: MouseEvent) => { e.stopPropagation(); onuninstall(); }}
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
        </svg>
        {$t("profile.uninstall")}
      </button>
    {:else}
      <button
        class="action-btn install"
        onclick={(e: MouseEvent) => { e.stopPropagation(); oninstall(); }}
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        {$t("profile.install")}
      </button>
    {/if}
  </div>
</div>

<style>
  .card {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    background: var(--bg-card);
    border: 2px solid var(--border);
    border-radius: var(--radius);
    transition: all var(--transition);
    flex: 1;
    min-width: 0;
    cursor: pointer;
  }
  .card:hover {
    border-color: var(--accent);
    background: var(--bg-tertiary);
  }
  .card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 10px var(--accent-glow);
  }
  .card-icon {
    width: 24px;
    height: 24px;
    flex-shrink: 0;
  }
  .card-icon :global(svg) {
    width: 100%;
    height: 100%;
  }
  .card-body {
    flex: 1;
    min-width: 0;
  }
  .card-top {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .card-name {
    font-weight: 600;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .status {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--success);
    flex-shrink: 0;
  }
  .ram-hint {
    font-size: 10px;
    font-weight: 500;
    color: var(--text-muted);
    flex-shrink: 0;
  }
  .ram-hint.insufficient {
    color: var(--danger);
    font-weight: 700;
  }
  .action-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 0;
    font-size: 11px;
    transition: all var(--transition);
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-muted);
  }
  .action-btn:hover {
    color: var(--text-primary);
  }
  .action-btn.install {
    color: var(--accent);
  }
  .action-btn.install:hover {
    filter: brightness(1.15);
  }
  .action-btn.uninstall:hover {
    color: var(--danger);
  }
</style>
