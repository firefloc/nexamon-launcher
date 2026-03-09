<script lang="ts">
  let { label = "", detail = "", progress = 0 }: { label: string; detail: string; progress: number } = $props();
  let pct = $derived(Math.round(progress * 100));
</script>

<div class="progress-container">
  <div class="progress-header">
    <span class="label">{label}</span>
    <span class="pct">{pct}%</span>
  </div>
  <div class="progress-track">
    <div class="progress-fill" style="width: {pct}%"></div>
  </div>
  {#if detail}
    <div class="detail">{detail}</div>
  {/if}
</div>

<style>
  .progress-container {
    width: 100%;
  }
  .progress-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 6px;
    font-size: 13px;
  }
  .label { color: var(--text-primary); }
  .pct { color: var(--text-secondary); }
  .progress-track {
    height: 8px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    overflow: hidden;
  }
  .progress-fill {
    height: 100%;
    background: var(--gradient-progress);
    border-radius: 999px;
    transition: width 0.3s ease;
    box-shadow: 0 0 12px rgba(72, 210, 198, 0.3);
  }
  .detail {
    margin-top: 4px;
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
