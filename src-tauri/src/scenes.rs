//! Named scenes (layer stacks) kept in `scenes.json` in the app's config folder, so later
//! features (per-game profiles) can use them without the window being open.

use crate::engine::layers::Scene;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SavedScene {
    pub name: String,
    pub scene: Scene,
}

pub fn load(path: &Path) -> Vec<SavedScene> {
    std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

/// Write through a temporary file so a crash never leaves a half-written list behind.
pub fn store(path: &Path, list: &[SavedScene]) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(list)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Add or replace (same name, ignoring case) a scene; keeps the list sorted by name.
pub fn upsert(list: &mut Vec<SavedScene>, name: &str, scene: Scene) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(anyhow!("scene name is empty"));
    }
    if name.chars().count() > 40 {
        return Err(anyhow!("scene name is longer than 40 characters"));
    }
    match list.iter_mut().find(|s| s.name.to_lowercase() == name.to_lowercase()) {
        Some(s) => s.scene = scene,
        None => list.push(SavedScene { name: name.to_string(), scene }),
    }
    list.sort_by_key(|s| s.name.to_lowercase());
    Ok(())
}

pub fn remove(list: &mut Vec<SavedScene>, name: &str) {
    list.retain(|s| s.name != name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::effects::SwEffect;

    fn scene(kind: &str) -> Scene {
        Scene::single(SwEffect { kind: kind.into(), speed: 1.0, brightness: 1.0, colours: vec![], direction: String::new(), scale: 1.0 })
    }

    #[test]
    fn upsert_replaces_by_name_and_sorts() {
        let mut list = Vec::new();
        upsert(&mut list, "Oyun", scene("wave")).unwrap();
        upsert(&mut list, "  çalışma ", scene("aurora")).unwrap();
        upsert(&mut list, "oyun", scene("fire")).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list.iter().find(|s| s.name == "Oyun").unwrap().scene, scene("fire"));
        assert!(upsert(&mut list, "   ", scene("wave")).is_err());
    }

    #[test]
    fn store_and_load_round_trip() {
        let dir = std::env::temp_dir().join(format!("prizma-test-{}", std::process::id()));
        let path = dir.join("scenes.json");
        let mut list = Vec::new();
        upsert(&mut list, "Gece", scene("stars")).unwrap();
        store(&path, &list).unwrap();
        assert_eq!(load(&path), list);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(load(&path).is_empty());
    }
}
