// ============================================================================
// Version Migration Module
// ============================================================================
//
// This module provides version migration support for serialized data,
// allowing older formats to be loaded and automatically upgraded to the
// current version.
//
// **Validates: Requirements 8.3**

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Version Information
// ============================================================================

/// Current serialization format version
pub const CURRENT_VERSION: u32 = 1;

/// Versioned wrapper for serialized data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Versioned<T> {
    /// Format version number
    pub version: u32,
    /// The actual data
    pub data: T,
}

impl<T> Versioned<T> {
    /// Create a new versioned value with the current version
    pub fn new(data: T) -> Self {
        Self {
            version: CURRENT_VERSION,
            data,
        }
    }

    /// Create a versioned value with a specific version (for testing)
    pub fn with_version(version: u32, data: T) -> Self {
        Self { version, data }
    }

    /// Get the version number
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Check if this is the current version
    pub fn is_current_version(&self) -> bool {
        self.version == CURRENT_VERSION
    }

    /// Consume the wrapper and return the inner data
    pub fn into_inner(self) -> T {
        self.data
    }
}

// ============================================================================
// Migration Errors
// ============================================================================

/// Errors that can occur during migration
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationError {
    pub from_version: u32,
    pub to_version: u32,
    pub kind: MigrationErrorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MigrationErrorKind {
    /// Version is too old and cannot be migrated
    UnsupportedVersion { reason: String },
    /// Version is newer than current (downgrade not supported)
    FutureVersion { reason: String },
    /// Migration failed due to data corruption
    DataCorruption { reason: String },
    /// Migration step failed
    MigrationFailed { step: String, reason: String },
}

impl fmt::Display for MigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Migration error (v{} -> v{}): {}",
            self.from_version, self.to_version, self.kind
        )
    }
}

impl fmt::Display for MigrationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrationErrorKind::UnsupportedVersion { reason } => {
                write!(f, "Unsupported version: {}", reason)
            }
            MigrationErrorKind::FutureVersion { reason } => {
                write!(f, "Future version: {}", reason)
            }
            MigrationErrorKind::DataCorruption { reason } => {
                write!(f, "Data corruption: {}", reason)
            }
            MigrationErrorKind::MigrationFailed { step, reason } => {
                write!(f, "Migration step '{}' failed: {}", step, reason)
            }
        }
    }
}

impl std::error::Error for MigrationError {}

// ============================================================================
// Migration Trait
// ============================================================================

/// Trait for types that support version migration
pub trait Migratable: Sized {
    /// Migrate from an older version to the current version
    ///
    /// # Arguments
    /// * `from_version` - The version of the source data
    /// * `data` - The serialized data as a JSON value
    ///
    /// # Returns
    /// The migrated data in the current format, or an error if migration fails
    fn migrate(from_version: u32, data: serde_json::Value) -> Result<Self, MigrationError>;

    /// Check if a version can be migrated
    fn can_migrate(from_version: u32) -> bool {
        from_version <= CURRENT_VERSION
    }

    /// Get the minimum supported version for migration
    fn min_supported_version() -> u32 {
        1
    }
}

// ============================================================================
// Migration Helper Functions
// ============================================================================

/// Deserialize and migrate versioned data from RON
pub fn from_ron_versioned<T>(s: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de> + Migratable,
{
    // First, try to parse as versioned data
    let ron_value: ron::Value = ron::from_str(s)
        .map_err(|e| format!("RON parse error: {}", e))?;
    
    let rust_value: serde_json::Value = ron_value
        .into_rust()
        .map_err(|e| format!("RON conversion error: {}", e))?;
    
    let value = rust_value;

    // Check if version field exists
    if let Some(version_value) = value.get("version") {
        let version = version_value
            .as_u64()
            .ok_or_else(|| "Version field must be a number".to_string())? as u32;

        // Get the data field
        let data = value.get("data")
            .ok_or_else(|| "Missing 'data' field in versioned format".to_string())?;

        if version == CURRENT_VERSION {
            // Current version, deserialize directly
            T::deserialize(data).map_err(|e| format!("Deserialization error: {}", e))
        } else if version < CURRENT_VERSION {
            // Old version, migrate
            T::migrate(version, data.clone()).map_err(|e| format!("Migration error: {}", e))
        } else {
            // Future version
            Err(format!(
                "Data is from a newer version (v{}) than current (v{}). \
                 Please upgrade the engine.",
                version, CURRENT_VERSION
            ))
        }
    } else {
        // No version field, assume version 1 (legacy format)
        T::migrate(1, value).map_err(|e| format!("Migration error: {}", e))
    }
}

/// Deserialize and migrate versioned data from binary
pub fn from_binary_versioned<T>(bytes: &[u8]) -> Result<T, String>
where
    T: for<'de> Deserialize<'de> + Migratable,
{
    // Binary format always includes version as first 4 bytes
    if bytes.len() < 4 {
        return Err("Binary data too short to contain version".to_string());
    }

    let version = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let data_bytes = &bytes[4..];

    if version == CURRENT_VERSION {
        // Current version, deserialize directly
        bincode::deserialize(data_bytes).map_err(|e| format!("Deserialization error: {}", e))
    } else if version < CURRENT_VERSION {
        // Old version, need to deserialize to JSON first, then migrate
        let value: serde_json::Value = bincode::deserialize(data_bytes)
            .map_err(|e| format!("Binary deserialization error: {}", e))?;
        T::migrate(version, value).map_err(|e| format!("Migration error: {}", e))
    } else {
        Err(format!(
            "Data is from a newer version (v{}) than current (v{}). \
             Please upgrade the engine.",
            version, CURRENT_VERSION
        ))
    }
}

/// Serialize data with version information to RON
pub fn to_ron_versioned<T>(value: &T) -> Result<String, String>
where
    T: Serialize,
{
    let versioned = Versioned::new(value);
    ron::to_string(&versioned).map_err(|e| format!("RON serialization error: {}", e))
}

/// Serialize data with version information to binary
pub fn to_binary_versioned<T>(value: &T) -> Result<Vec<u8>, String>
where
    T: Serialize,
{
    let mut result = Vec::new();
    
    // Write version as first 4 bytes
    result.extend_from_slice(&CURRENT_VERSION.to_le_bytes());
    
    // Write data
    let data_bytes =
        bincode::serialize(value).map_err(|e| format!("Binary serialization error: {}", e))?;
    result.extend_from_slice(&data_bytes);
    
    Ok(result)
}

// ============================================================================
// Migration Implementations for Core Types
// ============================================================================

impl Migratable for crate::Vec3 {
    fn migrate(from_version: u32, data: serde_json::Value) -> Result<Self, MigrationError> {
        if from_version > CURRENT_VERSION {
            return Err(MigrationError {
                from_version,
                to_version: CURRENT_VERSION,
                kind: MigrationErrorKind::FutureVersion {
                    reason: "Cannot downgrade from future version".to_string(),
                },
            });
        }

        // Version 1 is the current format, no migration needed
        serde_json::from_value(data).map_err(|e| MigrationError {
            from_version,
            to_version: CURRENT_VERSION,
            kind: MigrationErrorKind::DataCorruption {
                reason: format!("Failed to deserialize Vec3: {}", e),
            },
        })
    }
}

impl Migratable for crate::Quat {
    fn migrate(from_version: u32, data: serde_json::Value) -> Result<Self, MigrationError> {
        if from_version > CURRENT_VERSION {
            return Err(MigrationError {
                from_version,
                to_version: CURRENT_VERSION,
                kind: MigrationErrorKind::FutureVersion {
                    reason: "Cannot downgrade from future version".to_string(),
                },
            });
        }

        // Version 1 is the current format, no migration needed
        serde_json::from_value(data).map_err(|e| MigrationError {
            from_version,
            to_version: CURRENT_VERSION,
            kind: MigrationErrorKind::DataCorruption {
                reason: format!("Failed to deserialize Quat: {}", e),
            },
        })
    }
}

impl Migratable for crate::Transform {
    fn migrate(from_version: u32, data: serde_json::Value) -> Result<Self, MigrationError> {
        if from_version > CURRENT_VERSION {
            return Err(MigrationError {
                from_version,
                to_version: CURRENT_VERSION,
                kind: MigrationErrorKind::FutureVersion {
                    reason: "Cannot downgrade from future version".to_string(),
                },
            });
        }

        // Version 1 is the current format, no migration needed
        serde_json::from_value(data).map_err(|e| MigrationError {
            from_version,
            to_version: CURRENT_VERSION,
            kind: MigrationErrorKind::DataCorruption {
                reason: format!("Failed to deserialize Transform: {}", e),
            },
        })
    }
}

impl Migratable for crate::Color {
    fn migrate(from_version: u32, data: serde_json::Value) -> Result<Self, MigrationError> {
        if from_version > CURRENT_VERSION {
            return Err(MigrationError {
                from_version,
                to_version: CURRENT_VERSION,
                kind: MigrationErrorKind::FutureVersion {
                    reason: "Cannot downgrade from future version".to_string(),
                },
            });
        }

        // Version 1 is the current format, no migration needed
        serde_json::from_value(data).map_err(|e| MigrationError {
            from_version,
            to_version: CURRENT_VERSION,
            kind: MigrationErrorKind::DataCorruption {
                reason: format!("Failed to deserialize Color: {}", e),
            },
        })
    }
}

// ============================================================================
// Batch Migration Tools
// ============================================================================

/// Result of a batch migration operation
#[derive(Debug, Clone)]
pub struct BatchMigrationResult {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<(String, MigrationError)>,
}

impl BatchMigrationResult {
    pub fn new() -> Self {
        Self {
            total: 0,
            succeeded: 0,
            failed: 0,
            errors: Vec::new(),
        }
    }

    pub fn add_success(&mut self) {
        self.total += 1;
        self.succeeded += 1;
    }

    pub fn add_failure(&mut self, name: String, error: MigrationError) {
        self.total += 1;
        self.failed += 1;
        self.errors.push((name, error));
    }

    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.succeeded as f64 / self.total as f64
        }
    }
}

impl Default for BatchMigrationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BatchMigrationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Batch Migration Result:")?;
        writeln!(f, "  Total: {}", self.total)?;
        writeln!(f, "  Succeeded: {}", self.succeeded)?;
        writeln!(f, "  Failed: {}", self.failed)?;
        writeln!(f, "  Success Rate: {:.1}%", self.success_rate() * 100.0)?;
        
        if !self.errors.is_empty() {
            writeln!(f, "\nErrors:")?;
            for (name, error) in &self.errors {
                writeln!(f, "  {}: {}", name, error)?;
            }
        }
        
        Ok(())
    }
}

/// Migrate a batch of RON files
pub fn migrate_ron_files<T>(files: &[(String, String)]) -> BatchMigrationResult
where
    T: for<'de> Deserialize<'de> + Migratable + Serialize,
{
    let mut result = BatchMigrationResult::new();

    for (name, content) in files {
        match from_ron_versioned::<T>(content) {
            Ok(_data) => {
                result.add_success();
            }
            Err(e) => {
                // Try to extract version info from error
                let error = MigrationError {
                    from_version: 0,
                    to_version: CURRENT_VERSION,
                    kind: MigrationErrorKind::DataCorruption {
                        reason: e.to_string(),
                    },
                };
                result.add_failure(name.clone(), error);
            }
        }
    }

    result
}

/// Migrate a batch of binary files
pub fn migrate_binary_files<T>(files: &[(String, Vec<u8>)]) -> BatchMigrationResult
where
    T: for<'de> Deserialize<'de> + Migratable + Serialize,
{
    let mut result = BatchMigrationResult::new();

    for (name, bytes) in files {
        match from_binary_versioned::<T>(bytes) {
            Ok(_data) => {
                result.add_success();
            }
            Err(e) => {
                let error = MigrationError {
                    from_version: 0,
                    to_version: CURRENT_VERSION,
                    kind: MigrationErrorKind::DataCorruption {
                        reason: e.to_string(),
                    },
                };
                result.add_failure(name.clone(), error);
            }
        }
    }

    result
}
