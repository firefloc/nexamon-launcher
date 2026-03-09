<script lang="ts">
  import { updateAvailable, updateProgress, installUpdate, dismissUpdate } from "../lib/stores/updater";

  let installing = $derived($updateProgress !== "");
</script>

{#if $updateAvailable}
  <div class="update-banner">
    <div class="update-info">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
      </svg>
      <span>
        {#if installing}
          {$updateProgress}
        {:else}
          Update <strong>v{$updateAvailable.version}</strong> available
        {/if}
      </span>
    </div>
    <div class="update-actions">
      {#if !installing}
        <button class="update-btn" onclick={installUpdate}>Update now</button>
        <button class="dismiss-btn" onclick={dismissUpdate} title="Dismiss">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .update-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--accent);
    color: white;
    padding: 8px 16px;
    font-size: 13px;
    animation: slide-down 0.2s ease;
  }
  .update-info {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .update-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .update-btn {
    background: rgba(255, 255, 255, 0.2);
    color: white;
    padding: 4px 12px;
    border-radius: var(--radius);
    font-size: 12px;
    font-weight: 600;
    border: none;
    cursor: pointer;
    transition: background var(--transition);
  }
  .update-btn:hover {
    background: rgba(255, 255, 255, 0.3);
  }
  .dismiss-btn {
    background: none;
    color: rgba(255, 255, 255, 0.7);
    padding: 4px;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
  }
  .dismiss-btn:hover {
    color: white;
  }
  @keyframes slide-down {
    from { transform: translateY(-100%); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }
</style>
