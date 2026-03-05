use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
pub struct ProgressPayload {
    pub label: String,
    pub detail: String,
    pub progress: f64,
}

#[derive(Clone, Serialize)]
pub struct StatePayload {
    pub state: String,
}

#[derive(Clone, Serialize)]
pub struct LogPayload {
    pub line: String,
}

pub fn emit_progress(app: &AppHandle, label: &str, detail: &str, progress: f64) {
    let _ = app.emit("progress", ProgressPayload {
        label: label.to_string(),
        detail: detail.to_string(),
        progress,
    });
}

pub fn emit_state(app: &AppHandle, state: &str) {
    let _ = app.emit("launcher_state", StatePayload {
        state: state.to_string(),
    });
}

pub fn emit_log(app: &AppHandle, line: &str) {
    let _ = app.emit("game_log", LogPayload {
        line: line.to_string(),
    });
}
