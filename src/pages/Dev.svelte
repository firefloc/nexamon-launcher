<script lang="ts">
  import { t } from "../lib/i18n";
  import { debugOutlines } from "../lib/stores/dev";
</script>

<div class="dev-page">
  <div class="dev-header">
    <h2>{$t("dev.title")}</h2>
    <span class="dev-badge">DEV</span>
  </div>

  <div class="section">
    <h3>{$t("dev.layout_debug")}</h3>
    <label class="field checkbox">
      <input type="checkbox" bind:checked={$debugOutlines} />
      <span>{$t("dev.show_outlines")}</span>
    </label>
    <div class="legend">
      <div class="legend-item"><span class="swatch" style="border-color: red;"></span> main-page</div>
      <div class="legend-item"><span class="swatch" style="border-color: lime;"></span> hero-row</div>
      <div class="legend-item"><span class="swatch" style="border-color: cyan;"></span> selected-info</div>
      <div class="legend-item"><span class="swatch" style="border-color: orange;"></span> btn-row</div>
      <div class="legend-item"><span class="swatch" style="border-color: magenta;"></span> profiles</div>
    </div>
  </div>
</div>

<style>
  .dev-page {
    max-width: 700px;
  }
  .dev-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 24px;
  }
  h2 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 1px;
  }
  .dev-badge {
    background: var(--warning);
    color: #000;
    font-size: 10px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 4px;
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
  .field.checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
  }
  .field.checkbox input {
    width: 16px;
    height: 16px;
    accent-color: var(--warning);
  }
  .field.checkbox span {
    font-size: 13px;
    color: var(--text-secondary);
  }
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-top: 8px;
  }
  .legend-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .swatch {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px dashed;
    border-radius: 2px;
  }
</style>
