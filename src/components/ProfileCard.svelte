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

  const iconMap: Record<string, string> = {
    low: "🍃",
    high: "🔥",
    ultra: "⚡",
  };
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="card" class:selected role="button" tabindex="0" onclick={onselect} onkeydown={(e) => { if (e.key === 'Enter') onselect(); }}>
  <div class="card-icon">{iconMap[profile.icon] ?? "📦"}</div>
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
    <div class="selected-badge">✓</div>
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
    font-size: 28px;
    width: 40px;
    text-align: center;
  }
  .card-info {
    flex: 1;
  }
  .card-name {
    font-weight: 600;
    font-size: 14px;
    margin-bottom: 2px;
  }
  .card-desc {
    font-size: 12px;
    color: var(--text-secondary);
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
    color: var(--success, #00d2a0);
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
    color: var(--error, #e05050);
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
    font-size: 12px;
    font-weight: bold;
  }
</style>
