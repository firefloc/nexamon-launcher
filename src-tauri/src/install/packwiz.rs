use log::{debug, error, info, warn};
use reqwest::Client;
use std::path::Path;
use std::process::Stdio;
use tauri::AppHandle;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::config::paths;
use crate::util::progress::{emit_log, emit_progress};

const PACKWIZ_BOOTSTRAP_URL: &str =
    "https://github.com/packwiz/packwiz-installer-bootstrap/releases/latest/download/packwiz-installer-bootstrap.jar";

/// Pre-flight checks before running packwiz-installer.
/// Returns descriptive errors for each issue found.
async fn preflight_checks(
    client: &Client,
    java_path: &Path,
    pack_url: &str,
    instance_dir: &Path,
    packwiz_jar: &Path,
) -> Vec<String> {
    let mut issues = Vec::new();

    // Check java binary exists and is executable
    if !java_path.exists() {
        issues.push(format!("Java binary not found: {}", java_path.display()));
    } else {
        debug!("[packwiz] Java binary exists: {}", java_path.display());
        // Try running java -version
        match tokio::process::Command::new(java_path)
            .arg("-version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
        {
            Ok(output) => {
                let ver = String::from_utf8_lossy(&output.stderr);
                debug!("[packwiz] Java version: {}", ver.trim());
            }
            Err(e) => {
                issues.push(format!("Java not executable: {e}"));
            }
        }
    }

    // Check packwiz jar exists and has non-zero size
    if packwiz_jar.exists() {
        match std::fs::metadata(packwiz_jar) {
            Ok(meta) => {
                debug!(
                    "[packwiz] packwiz-installer-bootstrap.jar size: {} bytes",
                    meta.len()
                );
                if meta.len() == 0 {
                    issues.push("packwiz-installer-bootstrap.jar is empty (0 bytes)".into());
                }
            }
            Err(e) => {
                issues.push(format!("Cannot read packwiz-installer-bootstrap.jar metadata: {e}"));
            }
        }
    } else {
        debug!("[packwiz] packwiz-installer-bootstrap.jar not cached, will download");
    }

    // Check pack URL is reachable (HEAD request)
    debug!("[packwiz] Checking pack URL: {pack_url}");
    match client.head(pack_url).send().await {
        Ok(resp) => {
            debug!("[packwiz] Pack URL status: {}", resp.status());
            if !resp.status().is_success() {
                issues.push(format!(
                    "Pack URL returned HTTP {}: {pack_url}",
                    resp.status()
                ));
            }
        }
        Err(e) => {
            issues.push(format!("Pack URL unreachable: {e}"));
        }
    }

    // Check instance_dir exists or can be created
    if !instance_dir.exists() {
        debug!(
            "[packwiz] Instance dir does not exist yet, will create: {}",
            instance_dir.display()
        );
    } else {
        debug!("[packwiz] Instance dir exists: {}", instance_dir.display());
    }

    issues
}

pub async fn sync_mods(
    client: &Client,
    java_path: &Path,
    pack_url: &str,
    instance_dir: &Path,
    app: &AppHandle,
) -> Result<(), String> {
    info!("[packwiz] === sync_mods START ===");
    info!("[packwiz] java_path: {}", java_path.display());
    info!("[packwiz] pack_url: {pack_url}");
    info!("[packwiz] instance_dir: {}", instance_dir.display());

    let packwiz_jar = paths::packwiz_dir().join("packwiz-installer-bootstrap.jar");
    info!("[packwiz] packwiz_jar: {}", packwiz_jar.display());

    // Run pre-flight checks
    emit_log(app, "[packwiz] Running pre-flight checks...");
    let issues = preflight_checks(client, java_path, pack_url, instance_dir, &packwiz_jar).await;
    if !issues.is_empty() {
        for issue in &issues {
            error!("[packwiz] PRE-FLIGHT FAIL: {issue}");
            emit_log(app, &format!("[packwiz] PRE-FLIGHT FAIL: {issue}"));
        }
        return Err(format!(
            "Packwiz pre-flight failed:\n{}",
            issues.join("\n")
        ));
    }
    info!("[packwiz] Pre-flight checks passed");
    emit_log(app, "[packwiz] Pre-flight checks passed");

    // Download packwiz-installer if not present
    if !packwiz_jar.exists() {
        info!("[packwiz] Downloading packwiz-installer from {PACKWIZ_BOOTSTRAP_URL}");
        emit_progress(app, "Downloading packwiz-installer...", "", 0.0);
        emit_log(app, "[packwiz] Downloading packwiz-installer-bootstrap.jar...");
        super::super::download::client::download_file(
            client,
            PACKWIZ_BOOTSTRAP_URL,
            &packwiz_jar,
            Some(app),
            "Downloading packwiz-installer...",
        )
        .await?;
        // Verify download
        let meta = std::fs::metadata(&packwiz_jar).map_err(|e| e.to_string())?;
        info!(
            "[packwiz] Downloaded packwiz-installer-bootstrap.jar ({} bytes)",
            meta.len()
        );
        emit_log(
            app,
            &format!("[packwiz] Downloaded ({} bytes)", meta.len()),
        );
        if meta.len() == 0 {
            return Err("Downloaded packwiz-installer-bootstrap.jar is empty".into());
        }
    } else {
        let meta = std::fs::metadata(&packwiz_jar).map_err(|e| e.to_string())?;
        info!(
            "[packwiz] Using cached packwiz-installer-bootstrap.jar ({} bytes)",
            meta.len()
        );
    }

    // Ensure instance directory exists
    std::fs::create_dir_all(instance_dir).map_err(|e| {
        let msg = format!("Failed to create instance dir: {e}");
        error!("[packwiz] {msg}");
        msg
    })?;

    emit_progress(app, "Syncing mods...", "Running packwiz-installer", 0.5);

    // Build command
    let cmd_args = vec![
        "-Dpackwiz.acceptOptional=true".to_string(),
        "-jar".to_string(),
        packwiz_jar.to_string_lossy().to_string(),
        "-g".to_string(),
        "-s".to_string(),
        "client".to_string(),
        pack_url.to_string(),
    ];
    info!(
        "[packwiz] Spawning: {} {}",
        java_path.display(),
        cmd_args.join(" ")
    );
    emit_log(
        app,
        &format!(
            "[packwiz] CMD: {} {}",
            java_path.display(),
            cmd_args.join(" ")
        ),
    );

    // Run packwiz-installer
    let mut child = tokio::process::Command::new(java_path)
        .args(&cmd_args)
        .current_dir(instance_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            let msg = format!("Failed to spawn packwiz-installer: {e}");
            error!("[packwiz] {msg}");
            msg
        })?;

    info!(
        "[packwiz] Process spawned, PID: {:?}",
        child.id().unwrap_or(0)
    );
    emit_log(
        app,
        &format!("[packwiz] Process PID: {:?}", child.id().unwrap_or(0)),
    );

    // Stream stdout
    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                info!("[packwiz/stdout] {line}");
                emit_log(&app_clone, &format!("[packwiz/out] {line}"));
            }
            debug!("[packwiz] stdout stream ended");
        });
    }

    // Stream stderr
    if let Some(stderr) = child.stderr.take() {
        let app_clone = app.clone();
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                warn!("[packwiz/stderr] {line}");
                emit_log(&app_clone, &format!("[packwiz/err] {line}"));
            }
            debug!("[packwiz] stderr stream ended");
        });
    }

    // Wait for process with timeout (5 minutes)
    info!("[packwiz] Waiting for process to finish (timeout: 300s)...");
    let timeout_duration = std::time::Duration::from_secs(300);
    let status = match tokio::time::timeout(timeout_duration, child.wait()).await {
        Ok(result) => result.map_err(|e| {
            let msg = format!("Error waiting for packwiz-installer: {e}");
            error!("[packwiz] {msg}");
            msg
        })?,
        Err(_) => {
            error!("[packwiz] TIMEOUT after 300s — killing process");
            emit_log(app, "[packwiz] TIMEOUT after 300s — killing process");
            let _ = child.kill().await;
            return Err("packwiz-installer timed out after 5 minutes".into());
        }
    };

    info!("[packwiz] Process exited: {status}");
    emit_log(app, &format!("[packwiz] Exit status: {status}"));

    if !status.success() {
        let msg = format!("packwiz-installer exited with: {status}");
        error!("[packwiz] {msg}");
        return Err(msg);
    }

    // List mods directory after sync
    let mods_dir = instance_dir.join("mods");
    if mods_dir.exists() {
        match std::fs::read_dir(&mods_dir) {
            Ok(entries) => {
                let count = entries.count();
                info!("[packwiz] Mods synced — {count} files in mods/");
                emit_log(app, &format!("[packwiz] Done — {count} files in mods/"));
            }
            Err(e) => warn!("[packwiz] Could not list mods dir: {e}"),
        }
    } else {
        warn!("[packwiz] mods/ directory not found after sync");
        emit_log(app, "[packwiz] WARNING: mods/ directory not found after sync");
    }

    info!("[packwiz] === sync_mods END ===");
    emit_progress(app, "Mods synced", "", 1.0);
    Ok(())
}
