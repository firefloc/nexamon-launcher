<script lang="ts">
  import { startLogin, pollLogin, type DeviceCode } from "../lib/stores/auth";

  let deviceCode = $state<DeviceCode | null>(null);
  let error = $state("");
  let polling = $state(false);

  async function handleLogin() {
    error = "";
    try {
      deviceCode = await startLogin();
      polling = true;
      await pollLogin();
    } catch (e: any) {
      error = e?.toString() || "Login failed";
      polling = false;
      deviceCode = null;
    }
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
        <p>Open <a href={deviceCode.verification_uri} target="_blank">{deviceCode.verification_uri}</a></p>
        <p>and enter this code:</p>
        <div class="code">{deviceCode.user_code}</div>
        {#if polling}
          <p class="waiting">Waiting for login...</p>
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
  .device-code a {
    color: var(--accent);
    text-decoration: underline;
  }
  .code {
    font-size: 28px;
    font-weight: 700;
    letter-spacing: 4px;
    margin: 16px 0;
    padding: 12px;
    background: var(--bg-tertiary);
    border-radius: var(--radius);
    font-family: monospace;
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
