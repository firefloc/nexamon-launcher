<script lang="ts">
  import { startLogin, pollLogin, type DeviceCode } from "../lib/stores/auth";
  import { open } from "@tauri-apps/plugin-shell";

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
    <h1 class="title">NEXAMON</h1>
    <p class="subtitle">Minecraft Launcher</p>

    {#if !deviceCode}
      <button class="login-btn" onclick={handleLogin}>
        Sign in with Microsoft
      </button>
    {:else}
      <div class="device-code">
        <p>A browser window has opened. Enter this code:</p>
        <button class="code" onclick={copyCode} title="Click to copy">
          {deviceCode.user_code}
        </button>
        <p class="copy-hint">{copied ? "Copied!" : "Click code to copy"}</p>
        <button class="open-link" onclick={openLink}>
          Open login page again
        </button>
        {#if polling}
          <p class="waiting">Waiting for authentication...</p>
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
    background: var(--bg-primary);
  }
  .login-card {
    text-align: center;
    padding: 48px;
    background: var(--bg-secondary);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow);
    min-width: 380px;
  }
  .title {
    font-size: 36px;
    font-weight: 700;
    color: var(--accent);
    letter-spacing: 5px;
    margin-bottom: 4px;
  }
  .subtitle {
    color: var(--text-secondary);
    margin-bottom: 32px;
    font-size: 14px;
  }
  .login-btn {
    background: var(--accent);
    color: white;
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
