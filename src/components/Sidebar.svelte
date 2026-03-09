<script lang="ts">
  import { account, logout } from "../lib/stores/auth";

  let { page = $bindable("main") }: { page: "main" | "settings" | "console" } = $props();

  function formatUuid(raw: string): string {
    const h = raw.replace(/-/g, "");
    if (h.length !== 32) return raw;
    return `${h.slice(0,8)}-${h.slice(8,12)}-${h.slice(12,16)}-${h.slice(16,20)}-${h.slice(20)}`;
  }

  let avatarError = $state(false);
  let avatarUrl = $derived(
    $account ? `https://mc-heads.net/avatar/${formatUuid($account.uuid)}/32` : ""
  );

  async function handleLogout() {
    await logout();
  }
</script>

<nav class="sidebar">
  <div class="logo">
    <img src="/src/assets/nexamon-icon.png" alt="" class="logo-icon" />
    <h1><span class="t-teal">NEX</span><span class="t-gold">A</span><span class="t-coral">MON</span></h1>
  </div>

  <div class="nav-items">
    <button class="nav-btn" class:active={page === "main"} onclick={() => (page = "main")}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polygon points="5 3 19 12 5 21 5 3"/>
      </svg>
      <span>Play</span>
    </button>
    <button class="nav-btn" class:active={page === "settings"} onclick={() => (page = "settings")}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
      </svg>
      <span>Settings</span>
    </button>
    <button class="nav-btn" class:active={page === "console"} onclick={() => (page = "console")}>
      <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>
      </svg>
      <span>Console</span>
    </button>
  </div>

  <div class="user-section">
    {#if $account}
      <div class="user-info">
        {#if !avatarError}
          <img
            src={avatarUrl}
            alt="avatar"
            class="avatar"
            onerror={() => { avatarError = true; }}
          />
        {:else}
          <div class="avatar avatar-fallback">
            {$account.username.charAt(0).toUpperCase()}
          </div>
        {/if}
        <span class="username">{$account.username}</span>
      </div>
      <button class="logout-btn" onclick={handleLogout}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/>
        </svg>
        Logout
      </button>
    {/if}
  </div>
</nav>

<style>
  .sidebar {
    width: 200px;
    background: var(--gradient-panel);
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border-subtle);
    padding: 16px 0;
  }
  .logo {
    padding: 0 20px 20px;
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .logo-icon {
    width: 28px;
    height: 28px;
    border-radius: 4px;
  }
  .logo h1 {
    font-family: 'Oxanium', 'Trebuchet MS', sans-serif;
    font-size: 17px;
    font-weight: 700;
    letter-spacing: 3px;
  }
  .t-teal { color: #48d2c6; }
  .t-gold { color: #e4ba54; }
  .t-coral { color: #e4684c; }
  .nav-items {
    flex: 1;
    padding: 12px 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .nav-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: var(--radius);
    font-size: 14px;
    color: var(--text-secondary);
    transition: all var(--transition);
  }
  .nav-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }
  .nav-btn.active {
    background: rgba(72, 210, 198, 0.15);
    color: var(--accent);
    border-left: 3px solid var(--accent);
    padding-left: 9px;
  }
  .nav-icon {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }
  .user-section {
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .user-info {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .avatar {
    width: 28px;
    height: 28px;
    border-radius: 4px;
    image-rendering: pixelated;
  }
  .avatar-fallback {
    background: var(--accent);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    font-weight: 700;
  }
  .username {
    font-size: 13px;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .logout-btn {
    font-size: 12px;
    color: var(--text-muted);
    padding: 4px 0;
    text-align: left;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .logout-btn:hover {
    color: var(--danger);
  }
</style>
