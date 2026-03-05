<script lang="ts">
  import { settings, saveSettings, type Settings } from "../lib/stores/settings";

  let local = $state<Settings>({ ...$settings });
  let saved = $state(false);

  $effect(() => {
    local = { ...$settings };
  });

  async function handleSave() {
    await saveSettings(local);
    saved = true;
    setTimeout(() => (saved = false), 2000);
  }
</script>

<div class="settings-page">
  <h2>Settings</h2>

  <div class="section">
    <h3>Performance</h3>
    <label class="field">
      <span>RAM Allocation</span>
      <div class="ram-control">
        <input
          type="range"
          min="2048"
          max="16384"
          step="512"
          bind:value={local.ram_mb}
        />
        <span class="ram-value">{(local.ram_mb / 1024).toFixed(1)} GB</span>
      </div>
    </label>
  </div>

  <div class="section">
    <h3>Java</h3>
    <label class="field">
      <span>Java Path (leave empty for auto-detect)</span>
      <input
        type="text"
        placeholder="Auto-detect"
        bind:value={local.java_path}
      />
    </label>
  </div>

  <div class="section">
    <h3>Launcher</h3>
    <label class="field checkbox">
      <input type="checkbox" bind:checked={local.close_on_launch} />
      <span>Close launcher when game starts</span>
    </label>
  </div>

  <div class="actions">
    <button class="save-btn" onclick={handleSave}>
      {saved ? "Saved ✓" : "Save Settings"}
    </button>
  </div>
</div>

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
</style>
