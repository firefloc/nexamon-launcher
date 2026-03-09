<script lang="ts">
  import type { Profile } from "../lib/stores/profiles";

  let { profile, selected = false, installed = false, onselect, oninstall, onuninstall }: {
    profile: Profile;
    selected: boolean;
    installed: boolean;
    onselect: () => void;
    oninstall: () => void;
    onuninstall: () => void;
  } = $props();
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
  <div class="card-info">
    <div class="card-name">{profile.name}</div>
    <div class="card-desc">{profile.description}</div>
  </div>
  <div class="card-actions">
    {#if installed}
      <span class="status installed">Installed</span>
      <button
        class="action-btn uninstall"
        title="Uninstall pack"
        onclick={(e: MouseEvent) => { e.stopPropagation(); onuninstall(); }}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
        </svg>
      </button>
    {:else}
      <button
        class="action-btn install"
        title="Install pack"
        onclick={(e: MouseEvent) => { e.stopPropagation(); oninstall(); }}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        Install
      </button>
    {/if}
  </div>
  {#if selected}
    <div class="selected-badge">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="20 6 9 17 4 12"/>
      </svg>
    </div>
  {/if}
</div>

<style>
  .card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    background: var(--bg-card);
    border: 2px solid var(--border);
    border-radius: var(--radius-lg);
    text-align: left;
    transition: all var(--transition);
    position: relative;
    width: 100%;
    cursor: pointer;
  }
  .card:hover {
    border-color: var(--accent);
    background: var(--bg-tertiary);
  }
  .card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 12px var(--accent-glow);
  }
  .card-icon {
    width: 36px;
    height: 36px;
    flex-shrink: 0;
  }
  .card-icon :global(svg) {
    width: 100%;
    height: 100%;
  }
  .card-info {
    flex: 1;
    min-width: 0;
  }
  .card-name {
    font-weight: 600;
    font-size: 14px;
    margin-bottom: 2px;
  }
  .card-desc {
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .card-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-right: 8px;
  }
  .status {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .status.installed {
    color: var(--success);
  }
  .action-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 10px;
    border-radius: var(--radius);
    font-size: 12px;
    font-weight: 600;
    transition: all var(--transition);
    background: none;
    border: none;
    cursor: pointer;
  }
  .action-btn.install {
    background: var(--accent);
    color: white;
    padding: 6px 12px;
  }
  .action-btn.install:hover {
    filter: brightness(1.15);
  }
  .action-btn.uninstall {
    color: var(--text-muted);
    padding: 6px;
  }
  .action-btn.uninstall:hover {
    color: var(--danger);
    background: rgba(224, 80, 80, 0.1);
  }
  .selected-badge {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--accent);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
</style>
