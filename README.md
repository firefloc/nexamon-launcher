# Nexamon Launcher

A custom Minecraft modpack launcher built with Tauri v2 and Svelte 5. Designed for the Nexamon modpack, it handles Microsoft authentication, automatic Java management, Fabric mod loader installation, and modpack synchronization via packwiz.

## Features

- **Microsoft Authentication** -- Device code flow (OAuth 2.0) through Xbox Live and XSTS to obtain a Minecraft access token. Credentials are persisted locally and support token refresh.
- **Automatic Java Management** -- Detects installed Java runtimes or downloads one automatically.
- **Minecraft + Fabric Installation** -- Downloads vanilla Minecraft assets, libraries, and client jar, then installs the Fabric mod loader on top.
- **Modpack Sync via packwiz** -- Keeps mods up-to-date by running packwiz-installer against a remote `pack.toml` manifest before each launch.
- **Profile System** -- Multiple profiles with different pack URLs. Default profile points to the Nexamon pack.
- **Settings** -- Configurable RAM allocation (2-16 GB), custom Java path, close-on-launch toggle.
- **Live Console** -- Real-time game log streaming with color-coded log levels (INFO/WARN/ERROR) and auto-scroll.
- **Progress Reporting** -- Step-by-step progress bar during the install/launch pipeline (Java check, Minecraft download, Fabric install, mod sync, launch).

## Tech Stack

| Layer    | Technology |
|----------|------------|
| Frontend | Svelte 5, TypeScript, Vite 7 |
| Backend  | Rust, Tauri v2 |
| Auth     | Microsoft Identity Platform (Azure AD device code flow) |
| Mods     | packwiz-installer |
| Build    | Cargo (Rust), npm (frontend) |

## Prerequisites

- **Node.js** >= 18
- **Rust** >= 1.77.2 (with `cargo`)
- Tauri v2 system dependencies -- see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
- On Linux: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf` (distro-dependent)

## Development

```bash
# Install frontend dependencies
npm install

# Start in dev mode (opens the Tauri window with hot-reload)
npm run tauri dev
```

The Vite dev server runs on `http://localhost:1420`. The Tauri backend compiles and opens a native window pointing at it.

## Build

```bash
# Build a production binary (output in src-tauri/target/release/bundle/)
npm run tauri build
```

## Project Structure

```
nexamon-launcher/
  src/                      # Svelte frontend
    pages/                  # Login, Main, Settings, Console
    components/             # Sidebar, ProfileCard, ProgressBar
    lib/stores/             # Svelte stores (auth, settings, profiles, launcher)
  src-tauri/                # Rust backend (Tauri v2)
    src/
      auth/                 # Microsoft, Xbox Live, XSTS, Minecraft auth
      commands/             # Tauri command handlers (IPC)
      config/               # Paths, settings, accounts, profiles (JSON)
      download/             # HTTP client, Java, assets, libraries
      install/              # Fabric installer, packwiz sync
      launch/               # Classpath, JVM arguments, process management
      util/                 # Platform detection, hashing, progress events
```

## Status

In development. The Microsoft Azure AD app registration is set up and the full auth flow is implemented.

## License

MIT
