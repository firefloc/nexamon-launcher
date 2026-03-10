<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { settings, saveSettings, type Settings } from "../lib/stores/settings";
  import { t, locale, type Locale } from "../lib/i18n";

  let local = $state<Settings>({ ...$settings });
  let saved = $state(false);
  let maxRamMb = $state(16384);

  $effect(() => {
    const s = $settings;
    // Re-apply after maxRamMb is known to ensure slider position is correct
    local = { ...s, ram_mb: Math.min(s.ram_mb, maxRamMb || s.ram_mb) };
  });

  $effect(() => {
    invoke<number>("get_system_ram_mb").then((ram) => {
      const max = Math.floor(ram / 512) * 512;
      maxRamMb = max;
      // Force re-clamp local value to update slider position
      local = { ...$settings, ram_mb: Math.min($settings.ram_mb, max) };
    });
  });

  async function handleSave() {
    await saveSettings(local);
    saved = true;
    setTimeout(() => (saved = false), 2000);
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

  <div class="actions">
    <button class="save-btn" onclick={handleSave}>
      {saved ? $t("settings.saved") + " ✓" : $t("settings.save")}
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
</style>
