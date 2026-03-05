<script lang="ts">
  import type { Profile } from "../lib/stores/profiles";

  let { profile, selected = false, onselect }: {
    profile: Profile;
    selected: boolean;
    onselect: () => void;
  } = $props();
</script>

<button class="card" class:selected onclick={onselect}>
  <div class="card-icon">{profile.icon === "high" ? "🔥" : profile.icon === "low" ? "🍃" : "📦"}</div>
  <div class="card-info">
    <div class="card-name">{profile.name}</div>
    <div class="card-desc">{profile.description}</div>
  </div>
  {#if selected}
    <div class="selected-badge">✓</div>
  {/if}
</button>

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
