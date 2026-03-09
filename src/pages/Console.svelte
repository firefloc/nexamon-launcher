<script lang="ts">
  import { gameLog, clearLog } from "../lib/stores/launcher";
  import { t } from "../lib/i18n";

  let logContainer: HTMLDivElement;
  let autoScroll = $state(true);

  $effect(() => {
    if (autoScroll && logContainer && $gameLog.length) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  });

  function getLineClass(line: string): string {
    if (line.includes("ERROR") || line.includes("FATAL")) return "error";
    if (line.includes("WARN")) return "warn";
    if (line.includes("INFO")) return "info";
    return "";
  }
</script>

<div class="console-page">
  <div class="console-header">
    <h2>{$t("console.title")}</h2>
    <div class="controls">
      <label class="auto-scroll">
        <input type="checkbox" bind:checked={autoScroll} />
        <span>{$t("console.auto_scroll")}</span>
      </label>
      <button class="clear-btn" onclick={() => clearLog()}>{$t("console.clear")}</button>
    </div>
  </div>

  <div class="log-area" bind:this={logContainer}>
    {#each $gameLog as line, i}
      <div class="log-line {getLineClass(line)}">{line}</div>
    {/each}
    {#if $gameLog.length === 0}
      <div class="empty">{$t("console.empty")}</div>
    {/if}
  </div>
</div>

<style>
  .console-page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .console-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  h2 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 1px;
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .auto-scroll {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-muted);
    cursor: pointer;
  }
  .auto-scroll input {
    accent-color: var(--accent);
    width: 14px;
    height: 14px;
  }
  .clear-btn {
    font-size: 12px;
    color: var(--text-muted);
    padding: 4px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }
  .clear-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-muted);
  }
  .log-area {
    flex: 1;
    background: #0d0e1a;
    border-radius: var(--radius);
    padding: 12px;
    overflow-y: auto;
    font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
    font-size: 12px;
    line-height: 1.5;
  }
  .log-line {
    white-space: pre-wrap;
    word-break: break-all;
  }
  .log-line.error { color: var(--danger); }
  .log-line.warn { color: var(--warning); }
  .log-line.info { color: var(--text-secondary); }
  .empty {
    color: var(--text-muted);
    text-align: center;
    padding: 48px;
  }
</style>
