//! Preferences System (Vizia version)
//!
//! Provides persistence for editor preferences including panel sizes,
//! workspace layouts, and user settings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PanelSizePreferences {
    #[serde(with = "panel_sizes_serde")]
    pub sizes: HashMap<String, f32>,
}

mod panel_sizes_serde {
    use serde::ser::SerializeSeq;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::collections::HashMap;

    pub fn serialize<S>(map: &HashMap<String, f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(map.len()))?;
        for (key, value) in map {
            seq.serialize_element(&(key, value))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<(String, f32)> = Deserialize::deserialize(deserializer)?;
        Ok(vec.into_iter().collect())
    }
}

impl PanelSizePreferences {
    pub fn get(&self, panel_id: &str) -> Option<f32> {
        self.sizes.get(panel_id).copied()
    }

    pub fn set(&mut self, panel_id: &str, size: f32) {
        self.sizes.insert(panel_id.to_string(), size);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorPreferences {
    pub panel_sizes: PanelSizePreferences,
    pub theme: String,
    pub show_grid: bool,
    pub grid_size: f32,
}

impl EditorPreferences {
    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(contents) = fs::read_to_string(&path) {
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("luminara").join("preferences.json")
    }
}
