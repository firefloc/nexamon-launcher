use log::{debug, info, warn};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::Path;

const INTEGRITY_FILE: &str = "nexamon_integrity.json";

/// Manifest generated at build time, hosted alongside the pack on Pages.
#[derive(Deserialize, Default)]
struct IntegrityManifest {
    /// mod jar filename → sha256 of the jar
    #[serde(default)]
    mods: HashMap<String, String>,
    /// Paxi datapack filename → sha256
    #[serde(default)]
    datapacks: HashMap<String, String>,
    /// critical config path (relative to config/) → sha256
    #[serde(default)]
    critical_configs: HashMap<String, String>,
}

impl IntegrityManifest {
    fn is_empty(&self) -> bool {
        self.mods.is_empty() && self.datapacks.is_empty() && self.critical_configs.is_empty()
    }
}

/// Result of integrity verification.
#[derive(Default)]
pub struct IntegrityResult {
    pub extra_mods: Vec<String>,
    pub missing_mods: Vec<String>,
    pub extra_datapacks: Vec<String>,
    pub missing_datapacks: Vec<String>,
    pub tampered_critical: Vec<String>,
}

impl IntegrityResult {
    pub fn is_ok(&self) -> bool {
        self.extra_mods.is_empty()
            && self.missing_mods.is_empty()
            && self.extra_datapacks.is_empty()
            && self.missing_datapacks.is_empty()
            && self.tampered_critical.is_empty()
    }

    pub fn format_error(&self) -> String {
        let mut parts = Vec::new();

        if !self.extra_mods.is_empty() {
            let list = format_list(&self.extra_mods);
            parts.push(format!(
                "Unauthorized mods found (remove them from mods/):\n{list}"
            ));
        }

        if !self.missing_mods.is_empty() {
            let list = format_list(&self.missing_mods);
            parts.push(format!("Missing required mods:\n{list}"));
        }

        if !self.extra_datapacks.is_empty() {
            let list = format_list(&self.extra_datapacks);
            parts.push(format!(
                "Unauthorized datapacks found (remove them from config/paxi/datapacks/):\n{list}"
            ));
        }

        if !self.missing_datapacks.is_empty() {
            let list = format_list(&self.missing_datapacks);
            parts.push(format!("Missing required datapacks:\n{list}"));
        }

        if !self.tampered_critical.is_empty() {
            let list = format_list(&self.tampered_critical);
            parts.push(format!(
                "Critical configs have been modified (delete them to restore defaults on next sync):\n{list}"
            ));
        }

        parts.join("\n\n")
    }
}

fn format_list(items: &[String]) -> String {
    items
        .iter()
        .map(|s| format!("  - {s}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn hash_file(path: &Path) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    let hash = Sha256::digest(&data);
    Some(format!("{:x}", hash))
}

/// Check a directory for extra/missing files against an expected set.
fn check_dir(
    dir: &Path,
    expected: &HashMap<String, String>,
    extension_filter: Option<&str>,
) -> (Vec<String>, Vec<String>) {
    let mut extra = Vec::new();
    let mut missing = Vec::new();

    let expected_names: HashSet<&str> = expected.keys().map(|s| s.as_str()).collect();

    // Scan for extra files
    if dir.exists() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(ext) = extension_filter {
                    if !name.ends_with(ext) {
                        continue;
                    }
                }
                if !expected_names.contains(name.as_str()) {
                    extra.push(name);
                }
            }
        }
    }

    // Check for missing files
    for expected_name in expected.keys() {
        if !dir.join(expected_name).exists() {
            missing.push(expected_name.clone());
        }
    }

    extra.sort();
    missing.sort();
    (extra, missing)
}

/// Fetch integrity manifest from remote URL, fallback to local copy.
async fn fetch_manifest(client: &Client, url: &str, instance_dir: &Path) -> IntegrityManifest {
    // Try remote first (authoritative, can't be locally tampered)
    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(text) = resp.text().await {
                match serde_json::from_str::<IntegrityManifest>(&text) {
                    Ok(m) => {
                        debug!("[integrity] Loaded remote manifest from {url}");
                        return m;
                    }
                    Err(e) => warn!("[integrity] Failed to parse remote manifest: {e}"),
                }
            }
        }
        Ok(resp) => warn!("[integrity] Remote manifest HTTP {}", resp.status()),
        Err(e) => warn!("[integrity] Failed to fetch remote manifest: {e}"),
    }

    // Fallback to local copy (synced by packwiz)
    let local = instance_dir.join(INTEGRITY_FILE);
    if local.exists() {
        if let Ok(data) = std::fs::read_to_string(&local) {
            if let Ok(m) = serde_json::from_str::<IntegrityManifest>(&data) {
                info!("[integrity] Using local manifest fallback");
                return m;
            }
        }
    }

    warn!("[integrity] No manifest available, skipping integrity check");
    IntegrityManifest::default()
}

/// Verify instance integrity against the remote manifest.
pub async fn verify_integrity(
    client: &Client,
    pack_url: &str,
    instance_dir: &Path,
) -> IntegrityResult {
    let manifest_url = pack_url.replace("pack.toml", INTEGRITY_FILE);
    let manifest = fetch_manifest(client, &manifest_url, instance_dir).await;

    let mut result = IntegrityResult::default();

    // Skip if no manifest available (first install or offline)
    if manifest.is_empty() {
        return result;
    }

    // ── Mods: mods/*.jar ──
    let mods_dir = instance_dir.join("mods");
    let (extra, missing) = check_dir(&mods_dir, &manifest.mods, Some(".jar"));
    result.extra_mods = extra;
    result.missing_mods = missing;

    // ── Datapacks: config/paxi/datapacks/* ──
    let dp_dir = instance_dir.join("config").join("paxi").join("datapacks");
    let (extra, missing) = check_dir(&dp_dir, &manifest.datapacks, None);
    result.extra_datapacks = extra;
    result.missing_datapacks = missing;

    // ── Critical configs: hash comparison ──
    let config_dir = instance_dir.join("config");
    for (rel_path, expected_hash) in &manifest.critical_configs {
        let path = config_dir.join(rel_path);
        match hash_file(&path) {
            Some(actual_hash) if &actual_hash != expected_hash => {
                debug!("[integrity] Critical config tampered: {rel_path}");
                result.tampered_critical.push(rel_path.clone());
            }
            None => {
                debug!("[integrity] Critical config missing: {rel_path}");
                result.tampered_critical.push(rel_path.clone());
            }
            _ => {}
        }
    }
    result.tampered_critical.sort();

    info!(
        "[integrity] Check: {} extra mods, {} missing mods, {} extra dp, {} missing dp, {} tampered configs",
        result.extra_mods.len(),
        result.missing_mods.len(),
        result.extra_datapacks.len(),
        result.missing_datapacks.len(),
        result.tampered_critical.len()
    );

    result
}
