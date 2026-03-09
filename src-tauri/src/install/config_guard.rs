use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::Path;

const MANIFEST_FILE: &str = ".nexamon_pack_configs.json";
const BACKUP_DIR: &str = ".config_backup";
const CRITICAL_FILE: &str = "critical_configs.json";

/// Critical configs manifest shipped with the pack.
#[derive(Deserialize, Default)]
struct CriticalManifest {
    #[serde(default)]
    message: String,
    #[serde(default)]
    paths: Vec<String>,
}

/// Result of post-sync analysis.
#[derive(Serialize, Default)]
pub struct ConfigUpdateResult {
    /// Configs force-updated (critical). User is notified but cannot revert via dialog.
    pub critical: Vec<String>,
    /// Message explaining critical updates.
    pub critical_message: String,
    /// Configs the user modified that the pack also updated (optional).
    pub optional: Vec<String>,
}

/// SHA256 hash of a file's contents.
fn hash_file(path: &Path) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    let hash = Sha256::digest(&data);
    Some(format!("{:x}", hash))
}

/// Recursively collect file hashes under a directory.
fn collect_hashes(base: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if !base.exists() {
        return map;
    }
    collect_hashes_recursive(base, base, &mut map);
    map
}

fn collect_hashes_recursive(root: &Path, current: &Path, map: &mut HashMap<String, String>) {
    let Ok(entries) = std::fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_hashes_recursive(root, &path, map);
        } else if path.is_file() {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            if let Some(hash) = hash_file(&path) {
                map.insert(rel.to_string_lossy().to_string(), hash);
            }
        }
    }
}

/// Copy a directory recursively.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("mkdir {}: {e}", dst.display()))?;
    let entries =
        std::fs::read_dir(src).map_err(|e| format!("read_dir {}: {e}", src.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest)
                .map_err(|e| format!("copy {} → {}: {e}", path.display(), dest.display()))?;
        }
    }
    Ok(())
}

/// Load the stored pack config manifest.
fn load_manifest(instance_dir: &Path) -> HashMap<String, String> {
    let path = instance_dir.join(MANIFEST_FILE);
    if !path.exists() {
        return HashMap::new();
    }
    let data = std::fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or_default()
}

/// Save the pack config manifest.
fn save_manifest(instance_dir: &Path, manifest: &HashMap<String, String>) {
    let path = instance_dir.join(MANIFEST_FILE);
    let data = serde_json::to_string_pretty(manifest).unwrap_or_default();
    if let Err(e) = std::fs::write(&path, data) {
        warn!("[config_guard] Failed to save manifest: {e}");
    }
}

/// Load the critical configs manifest from the pack.
fn load_critical_manifest(instance_dir: &Path) -> CriticalManifest {
    let path = instance_dir.join(CRITICAL_FILE);
    if !path.exists() {
        return CriticalManifest::default();
    }
    let data = std::fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or_default()
}

/// Check if a config path matches any critical pattern.
/// Supports exact paths and prefix patterns ending with /.
fn is_critical(file: &str, critical_paths: &HashSet<String>) -> bool {
    // Exact match
    if critical_paths.contains(file) {
        return true;
    }
    // Prefix match: "config/cobblemon/" matches "config/cobblemon/settings.json"
    for pattern in critical_paths {
        if pattern.ends_with('/') && file.starts_with(pattern.as_str()) {
            return true;
        }
    }
    false
}

/// Pre-sync: backup config/ and detect user-modified files.
pub fn pre_sync(instance_dir: &Path) -> Result<Vec<String>, String> {
    let config_dir = instance_dir.join("config");
    let backup_dir = instance_dir.join(BACKUP_DIR);

    if !config_dir.exists() {
        info!("[config_guard] No config/ dir, first install");
        return Ok(vec![]);
    }

    let manifest = load_manifest(instance_dir);
    if manifest.is_empty() {
        info!("[config_guard] No manifest, first tracked sync");
    }

    let current = collect_hashes(&config_dir);

    let mut user_modified = Vec::new();
    for (file, current_hash) in &current {
        match manifest.get(file) {
            Some(pack_hash) if pack_hash != current_hash => {
                user_modified.push(file.clone());
            }
            None => {
                user_modified.push(file.clone());
            }
            _ => {}
        }
    }

    if backup_dir.exists() {
        std::fs::remove_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    }
    copy_dir_recursive(&config_dir, &backup_dir)?;
    debug!(
        "[config_guard] Backed up config/ ({} files, {} user-modified)",
        current.len(),
        user_modified.len()
    );

    Ok(user_modified)
}

/// Post-sync: detect conflicts and split into critical vs optional.
pub fn post_sync(
    instance_dir: &Path,
    user_modified: &[String],
) -> Result<ConfigUpdateResult, String> {
    let config_dir = instance_dir.join("config");
    let new_hashes = collect_hashes(&config_dir);
    let old_manifest = load_manifest(instance_dir);

    // Load critical config paths from pack
    let critical_manifest = load_critical_manifest(instance_dir);
    let critical_paths: HashSet<String> = critical_manifest.paths.into_iter().collect();

    // Find conflicts
    let mut all_conflicts = Vec::new();
    for file in user_modified {
        let old_pack_hash = old_manifest.get(file);
        let new_pack_hash = new_hashes.get(file);

        match (old_pack_hash, new_pack_hash) {
            (Some(old), Some(new)) if old != new => {
                all_conflicts.push(file.clone());
            }
            _ => {}
        }
    }

    // Split into critical and optional
    let mut result = ConfigUpdateResult::default();
    for file in all_conflicts {
        if is_critical(&file, &critical_paths) {
            result.critical.push(file);
        } else {
            result.optional.push(file);
        }
    }
    result.critical_message = critical_manifest.message;

    // Save new manifest
    save_manifest(instance_dir, &new_hashes);

    info!(
        "[config_guard] Post-sync: {} critical, {} optional conflicts",
        result.critical.len(),
        result.optional.len()
    );

    Ok(result)
}

/// Restore only optional (non-critical) user configs from backup.
/// Critical configs stay as the pack version.
pub fn restore_user_configs(instance_dir: &Path) -> Result<u32, String> {
    let config_dir = instance_dir.join("config");
    let backup_dir = instance_dir.join(BACKUP_DIR);

    if !backup_dir.exists() {
        return Ok(0);
    }

    // Load critical paths to know which files to NOT restore
    let critical_manifest = load_critical_manifest(instance_dir);
    let critical_paths: HashSet<String> = critical_manifest.paths.into_iter().collect();

    // Restore all backup files EXCEPT critical ones
    let backup_hashes = collect_hashes(&backup_dir);
    let mut restored = 0u32;

    for (rel_path, _) in &backup_hashes {
        if is_critical(rel_path, &critical_paths) {
            debug!("[config_guard] Skipping critical config restore: {rel_path}");
            continue;
        }
        let src = backup_dir.join(rel_path);
        let dst = config_dir.join(rel_path);
        if let Some(parent) = dst.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::copy(&src, &dst).is_ok() {
            restored += 1;
        }
    }

    info!("[config_guard] Restored {restored} config files from backup (critical configs preserved)");
    cleanup_backup(instance_dir);
    Ok(restored)
}

/// Accept new pack configs. Just clean up the backup.
pub fn accept_new_configs(instance_dir: &Path) {
    info!("[config_guard] User accepted new configs");
    cleanup_backup(instance_dir);
}

/// Remove backup directory.
fn cleanup_backup(instance_dir: &Path) {
    let backup_dir = instance_dir.join(BACKUP_DIR);
    if backup_dir.exists() {
        let _ = std::fs::remove_dir_all(&backup_dir);
    }
}
