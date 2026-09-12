use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::registry::Registry;
use crate::{Error, Result};

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(default, rename = "Actions")]
    actions: Vec<ManifestAction>,
}

#[derive(Debug, Deserialize)]
struct ManifestAction {
    #[serde(default, rename = "UUID")]
    uuid: String,
}

/// Compare registered action UUIDs with `manifest.json`.
pub fn check(registry: &Registry, plugin_uuid: &str) -> Result<()> {
    let Some(path) = find_manifest(plugin_uuid) else {
        tracing::debug!("manifest.json not found; skipping UUID check");
        return Ok(());
    };
    let text = std::fs::read_to_string(&path)?;
    let manifest: Manifest = serde_json::from_str(&text)?;
    let registered: Vec<String> = registry
        .uuids()
        .into_iter()
        .map(|uuid| uuid.to_ascii_lowercase())
        .collect();
    let declared: Vec<String> = manifest
        .actions
        .iter()
        .map(|action| action.uuid.to_ascii_lowercase())
        .filter(|uuid| !uuid.is_empty())
        .collect();

    let missing: Vec<_> = declared
        .iter()
        .filter(|uuid| !registered.contains(uuid))
        .cloned()
        .collect();
    let extra: Vec<_> = registered
        .iter()
        .filter(|uuid| !declared.contains(uuid))
        .cloned()
        .collect();

    if missing.is_empty() && extra.is_empty() {
        return Ok(());
    }

    let message = format!(
        "manifest UUID mismatch ({}): missing in code {:?}, extra in code {:?}",
        path.display(),
        missing,
        extra
    );
    if cfg!(debug_assertions) {
        Err(Error::Manifest(message))
    } else {
        tracing::warn!("{message}");
        Ok(())
    }
}

fn find_manifest(plugin_uuid: &str) -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("manifest.json"),
        PathBuf::from(format!("{plugin_uuid}.sdPlugin/manifest.json")),
        PathBuf::from("..").join("manifest.json"),
    ];
    candidates
        .into_iter()
        .find(|path| path.exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| search_parents(&exe, "manifest.json"))
        })
}

fn search_parents(start: &Path, file_name: &str) -> Option<PathBuf> {
    let mut current = start.parent()?;
    for _ in 0..6 {
        let candidate = current.join(file_name);
        if candidate.exists() {
            return Some(candidate);
        }
        current = current.parent()?;
    }
    None
}
