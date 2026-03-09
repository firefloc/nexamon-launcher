<script lang="ts">
  import { startLogin, pollLogin, type DeviceCode } from "../lib/stores/auth";
  import { open } from "@tauri-apps/plugin-shell";
  import { t } from "../lib/i18n";

  let deviceCode = $state<DeviceCode | null>(null);
  let error = $state("");
  let polling = $state(false);
  let copied = $state(false);

  async function handleLogin() {
    error = "";
    try {
      deviceCode = await startLogin();
      polling = true;

      // Auto-open browser and copy code to clipboard
      try {
        await navigator.clipboard.writeText(deviceCode.user_code);
        copied = true;
        setTimeout(() => copied = false, 3000);
      } catch { /* clipboard may fail silently */ }

      try {
        await open(deviceCode.verification_uri);
      } catch { /* browser open may fail silently */ }

      await pollLogin();
    } catch (e: any) {
      error = e?.toString() || "Login failed";
      polling = false;
      deviceCode = null;
    }
  }

  async function copyCode() {
    if (!deviceCode) return;
    try {
      await navigator.clipboard.writeText(deviceCode.user_code);
      copied = true;
      setTimeout(() => copied = false, 2000);
    } catch {}
  }

  async function openLink() {
    if (!deviceCode) return;
    try {
      await open(deviceCode.verification_uri);
    } catch {}
  }
</script>

<div class="login-page">
  <div class="login-card">
    <img src="/src/assets/nexamon-icon.png" alt="Nexamon" class="login-icon" />
    <h1 class="title"><span class="t-teal">NEX</span><span class="t-gold">A</span><span class="t-coral">MON</span></h1>
    <p class="subtitle">{$t("login.subtitle")}</p>

    {#if !deviceCode}
      <button class="login-btn" onclick={handleLogin}>
        {$t("login.button")}
      </button>
    {:else}
      <div class="device-code">
        <p>{$t("login.browser_opened")}</p>
        <button class="code" onclick={copyCode} title={$t("login.copy_hint")}>
          {deviceCode.user_code}
        </button>
        <p class="copy-hint">{copied ? $t("login.copied") : $t("login.copy_hint")}</p>
        <button class="open-link" onclick={openLink}>
          {$t("login.open_again")}
        </button>
        {#if polling}
          <p class="waiting">{$t("login.waiting")}</p>
        {/if}
      </div>
    {/if}

    {#if error}
      <p class="error">{error}</p>
    {/if}
  </div>
</div>

<style>
  .login-page {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    background: var(--gradient-shell);
  }
  .login-card {
    text-align: center;
    padding: 48px;
    background: var(--gradient-panel);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-deep);
    min-width: 380px;
  }
  .login-icon {
    width: 80px;
    height: 80px;
    margin-bottom: 16px;
  }
  .title {
    font-family: 'Oxanium', 'Trebuchet MS', sans-serif;
    font-size: 36px;
    font-weight: 700;
    letter-spacing: 5px;
    margin-bottom: 4px;
  }
  .t-teal { color: #48d2c6; }
  .t-gold { color: #e4ba54; }
  .t-coral { color: #e4684c; }
  .subtitle {
    color: var(--text-secondary);
    margin-bottom: 32px;
    font-size: 14px;
    font-family: 'IBM Plex Mono', monospace;
    letter-spacing: 0.04em;
  }
  .login-btn {
    background: var(--accent);
    color: #081824;
    padding: 12px 32px;
    border-radius: var(--radius);
    font-size: 15px;
    font-weight: 600;
    transition: background var(--transition);
  }
  .login-btn:hover {
    background: var(--accent-hover);
  }
  .device-code {
    margin-top: 8px;
  }
  .code {
    display: block;
    width: 100%;
    font-size: 28px;
    font-weight: 700;
    letter-spacing: 4px;
    margin: 16px 0 4px;
    padding: 12px;
    background: var(--bg-tertiary);
    border-radius: var(--radius);
    font-family: monospace;
    color: var(--text-primary);
    cursor: pointer;
    border: 2px solid transparent;
    transition: border-color var(--transition);
  }
  .code:hover {
    border-color: var(--accent);
  }
  .copy-hint {
    font-size: 12px;
    color: var(--text-muted);
    margin-bottom: 12px;
  }
  .open-link {
    background: none;
    color: var(--accent);
    text-decoration: underline;
    font-size: 13px;
    padding: 4px 8px;
    cursor: pointer;
  }
  .open-link:hover {
    color: var(--accent-hover);
  }
  .waiting {
    color: var(--text-muted);
    font-size: 13px;
    animation: pulse 1.5s ease-in-out infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 0.5; }
    50% { opacity: 1; }
  }
  .error {
    color: var(--danger);
    margin-top: 16px;
    font-size: 13px;
  }
</style>
