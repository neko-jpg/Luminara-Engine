//! Preferences System
//!
//! Provides persistence for editor preferences including panel sizes,
//! workspace layouts, and user settings.
//!
//! **Validates Requirements:**
//! - 9.4: Panel sizes are persisted to user preferences
//!
//! # Storage Format
//!
//! Preferences are stored in JSON format in the user's config directory:
//! - Windows: `%APPDATA%\Luminara\preferences.json`
//! - macOS: `~/Library/Application Support/Luminara/preferences.json`
//! - Linux: `~/.config/luminara/preferences.json`

use gpui::{Pixels, px};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Panel size preferences
///
/// Stores the size of each panel identified by a unique panel ID.
/// Panel IDs should be stable across sessions (e.g., "scene_builder.hierarchy",
/// "scene_builder.inspector", etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PanelSizePreferences {
    /// Map of panel ID to size in pixels
    #[serde(with = "panel_sizes_serde")]
    pub sizes: HashMap<String, f32>,
}

/// Custom serialization for HashMap<String, Pixels>
mod panel_sizes_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(
        sizes: &HashMap<String, f32>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        sizes.serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<String, f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        HashMap::<String, f32>::deserialize(deserializer)
    }
}

impl PanelSizePreferences {
    /// Create a new empty preferences instance
    pub fn new() -> Self {
        Self {
            sizes: HashMap::new(),
        }
    }

    /// Get the size for a panel by ID
    ///
    /// Returns `None` if no size is stored for this panel.
    pub fn get_size(&self, panel_id: &str) -> Option<Pixels> {
        self.sizes.get(panel_id).map(|&size| px(size))
    }

    /// Set the size for a panel by ID
    ///
    /// # Arguments
    ///
    /// * `panel_id` - Unique identifier for the panel
    /// * `size` - Size in pixels to store
    pub fn set_size(&mut self, panel_id: String, size: Pixels) {
        // Use unsafe to extract the f32 value from Pixels
        let size_f32 = unsafe { std::mem::transmute::<Pixels, f32>(size) };
        self.sizes.insert(panel_id, size_f32);
    }

    /// Remove a panel size preference
    pub fn remove_size(&mut self, panel_id: &str) -> Option<Pixels> {
        self.sizes.remove(panel_id).map(px)
    }

    /// Clear all panel size preferences
    pub fn clear(&mut self) {
        self.sizes.clear();
    }

    /// Get the number of stored panel sizes
    pub fn len(&self) -> usize {
        self.sizes.len()
    }

    /// Check if there are no stored panel sizes
    pub fn is_empty(&self) -> bool {
        self.sizes.is_empty()
    }
}

/// Editor preferences
///
/// Contains all user preferences for the editor including panel sizes,
/// theme settings, keyboard shortcuts, etc.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorPreferences {
    /// Panel size preferences
    pub panel_sizes: PanelSizePreferences,
}

impl EditorPreferences {
    /// Create a new preferences instance with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the preferences file path
    ///
    /// Returns the platform-specific path where preferences are stored.
    pub fn preferences_path() -> Result<PathBuf, PreferencesError> {
        let config_dir = dirs::config_dir()
            .ok_or(PreferencesError::ConfigDirNotFound)?;
        
        let luminara_dir = config_dir.join("luminara");
        Ok(luminara_dir.join("preferences.json"))
    }

    /// Load preferences from disk
    ///
    /// If the preferences file doesn't exist, returns default preferences.
    /// If the file exists but is invalid, returns an error.
    ///
    /// # Requirements
    /// - Requirement 9.4: Load panel sizes on startup
    pub fn load() -> Result<Self, PreferencesError> {
        let path = Self::preferences_path()?;
        
        if !path.exists() {
            // No preferences file yet, return defaults
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)
            .map_err(|e| PreferencesError::ReadError(e.to_string()))?;
        
        let prefs: EditorPreferences = serde_json::from_str(&contents)
            .map_err(|e| PreferencesError::ParseError(e.to_string()))?;
        
        Ok(prefs)
    }

    /// Save preferences to disk
    ///
    /// Creates the config directory if it doesn't exist.
    ///
    /// # Requirements
    /// - Requirement 9.4: Save panel sizes to preferences
    pub fn save(&self) -> Result<(), PreferencesError> {
        let path = Self::preferences_path()?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PreferencesError::WriteError(e.to_string()))?;
        }

        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| PreferencesError::SerializeError(e.to_string()))?;
        
        fs::write(&path, contents)
            .map_err(|e| PreferencesError::WriteError(e.to_string()))?;
        
        Ok(())
    }

    /// Get panel size preference
    pub fn get_panel_size(&self, panel_id: &str) -> Option<Pixels> {
        self.panel_sizes.get_size(panel_id)
    }

    /// Set panel size preference
    pub fn set_panel_size(&mut self, panel_id: String, size: Pixels) {
        self.panel_sizes.set_size(panel_id, size);
    }
}

/// Errors that can occur when working with preferences
#[derive(Debug, Clone)]
pub enum PreferencesError {
    /// Could not find the config directory
    ConfigDirNotFound,
    /// Error reading the preferences file
    ReadError(String),
    /// Error parsing the preferences file
    ParseError(String),
    /// Error serializing preferences
    SerializeError(String),
    /// Error writing the preferences file
    WriteError(String),
}

impl std::fmt::Display for PreferencesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreferencesError::ConfigDirNotFound => {
                write!(f, "Could not find config directory")
            }
            PreferencesError::ReadError(msg) => {
                write!(f, "Error reading preferences: {}", msg)
            }
            PreferencesError::ParseError(msg) => {
                write!(f, "Error parsing preferences: {}", msg)
            }
            PreferencesError::SerializeError(msg) => {
                write!(f, "Error serializing preferences: {}", msg)
            }
            PreferencesError::WriteError(msg) => {
                write!(f, "Error writing preferences: {}", msg)
            }
        }
    }
}

impl std::error::Error for PreferencesError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_size_preferences() {
        let mut prefs = PanelSizePreferences::new();
        
        // Initially empty
        assert!(prefs.is_empty());
        assert_eq!(prefs.len(), 0);
        
        // Set a size
        prefs.set_size("test_panel".to_string(), px(320.0));
        assert_eq!(prefs.len(), 1);
        assert!(!prefs.is_empty());
        
        // Get the size
        let size = prefs.get_size("test_panel");
        assert_eq!(size, Some(px(320.0)));
        
        // Get non-existent size
        let missing = prefs.get_size("missing_panel");
        assert_eq!(missing, None);
        
        // Remove a size
        let removed = prefs.remove_size("test_panel");
        assert_eq!(removed, Some(px(320.0)));
        assert!(prefs.is_empty());
        
        // Clear all
        prefs.set_size("panel1".to_string(), px(100.0));
        prefs.set_size("panel2".to_string(), px(200.0));
        assert_eq!(prefs.len(), 2);
        prefs.clear();
        assert!(prefs.is_empty());
    }

    #[test]
    fn test_editor_preferences() {
        let mut prefs = EditorPreferences::new();
        
        // Set panel size
        prefs.set_panel_size("hierarchy".to_string(), px(260.0));
        prefs.set_panel_size("inspector".to_string(), px(320.0));
        
        // Get panel sizes
        assert_eq!(prefs.get_panel_size("hierarchy"), Some(px(260.0)));
        assert_eq!(prefs.get_panel_size("inspector"), Some(px(320.0)));
        assert_eq!(prefs.get_panel_size("missing"), None);
    }

    #[test]
    fn test_serialization() {
        let mut prefs = EditorPreferences::new();
        prefs.set_panel_size("panel1".to_string(), px(100.0));
        prefs.set_panel_size("panel2".to_string(), px(200.0));
        
        // Serialize
        let json = serde_json::to_string(&prefs).unwrap();
        
        // Deserialize
        let loaded: EditorPreferences = serde_json::from_str(&json).unwrap();
        
        // Verify
        assert_eq!(loaded.get_panel_size("panel1"), Some(px(100.0)));
        assert_eq!(loaded.get_panel_size("panel2"), Some(px(200.0)));
    }

    #[test]
    fn test_preferences_path() {
        let path = EditorPreferences::preferences_path();
        assert!(path.is_ok());
        
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("luminara"));
        assert!(path.to_string_lossy().ends_with("preferences.json"));
    }
}
