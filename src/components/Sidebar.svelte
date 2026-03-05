<script lang="ts">
  import { account, logout } from "../lib/stores/auth";

  let { page = $bindable("main") }: { page: "main" | "settings" | "console" } = $props();

  const nav = [
    { id: "main" as const, label: "Play", icon: "▶" },
    { id: "settings" as const, label: "Settings", icon: "⚙" },
    { id: "console" as const, label: "Console", icon: ">" },
  ];

  async function handleLogout() {
    await logout();
  }
</script>

<nav class="sidebar">
  <div class="logo">
    <h1>NEXAMON</h1>
  </div>

  <div class="nav-items">
    {#each nav as item}
      <button
        class="nav-btn"
        class:active={page === item.id}
        onclick={() => (page = item.id)}
      >
        <span class="icon">{item.icon}</span>
        <span>{item.label}</span>
      </button>
    {/each}
  </div>

  <div class="user-section">
    {#if $account}
      <div class="user-info">
        <img
          src="https://crafatar.com/avatars/{$account.uuid}?size=32&overlay"
          alt="avatar"
          class="avatar"
        />
        <span class="username">{$account.username}</span>
      </div>
      <button class="logout-btn" onclick={handleLogout}>Logout</button>
    {/if}
  </div>
</nav>

<style>
  .sidebar {
    width: 200px;
    background: var(--bg-secondary);
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border);
    padding: 16px 0;
  }
  .logo {
    padding: 0 20px 20px;
    border-bottom: 1px solid var(--border);
  }
  .logo h1 {
    font-size: 20px;
    font-weight: 700;
    color: var(--accent);
    letter-spacing: 3px;
  }
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
    background: var(--accent);
    color: white;
  }
  .icon {
    font-size: 16px;
    width: 20px;
    text-align: center;
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
  }
  .username {
    font-size: 13px;
    font-weight: 500;
  }
  .logout-btn {
    font-size: 12px;
    color: var(--text-muted);
    padding: 4px 0;
    text-align: left;
  }
  .logout-btn:hover {
    color: var(--danger);
  }
</style>
