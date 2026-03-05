use std::path::Path;
use std::process::Stdio;
use tauri::AppHandle;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::util::progress::{emit_log, emit_state};

pub async fn launch_minecraft(
    java_path: &Path,
    main_class: &str,
    jvm_args: &[String],
    game_args: &[String],
    game_dir: &Path,
    app: &AppHandle,
) -> Result<(), String> {
    std::fs::create_dir_all(game_dir).map_err(|e| e.to_string())?;

    let mut cmd = tokio::process::Command::new(java_path);
    cmd.args(jvm_args);
    cmd.arg(main_class);
    cmd.args(game_args);
    cmd.current_dir(game_dir);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Failed to launch: {e}"))?;

    emit_state(app, "running");

    // Stream stdout
    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                emit_log(&app_clone, &line);
            }
        });
    }

    // Stream stderr
    if let Some(stderr) = child.stderr.take() {
        let app_clone = app.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                emit_log(&app_clone, &line);
            }
        });
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;
    emit_state(app, "idle");
    emit_log(app, &format!("Game exited with code: {}", status));

    Ok(())
}
