use crate::app::App;

/// Represents a plugin dependency with optional version constraint
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginDependency {
    /// Name of the required plugin
    pub name: String,
    /// Optional version constraint (e.g., ">=1.0.0", "^2.0")
    pub version: Option<String>,
}

impl PluginDependency {
    /// Create a new plugin dependency without version constraint
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
        }
    }

    /// Create a new plugin dependency with version constraint
    pub fn with_version(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: Some(version.into()),
        }
    }
}

/// Error type for plugin loading failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginError {
    /// One or more dependencies are missing
    MissingDependencies {
        plugin_name: String,
        missing: Vec<PluginDependency>,
    },
    /// A dependency version constraint is not satisfied
    VersionMismatch {
        plugin_name: String,
        dependency: String,
        required: String,
        found: String,
    },
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::MissingDependencies {
                plugin_name,
                missing,
            } => {
                write!(
                    f,
                    "Plugin '{}' has unsatisfied dependencies: ",
                    plugin_name
                )?;
                for (i, dep) in missing.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "'{}'", dep.name)?;
                    if let Some(version) = &dep.version {
                        write!(f, " ({})", version)?;
                    }
                }
                Ok(())
            }
            PluginError::VersionMismatch {
                plugin_name,
                dependency,
                required,
                found,
            } => {
                write!(
                    f,
                    "Plugin '{}' requires '{}' version {}, but found version {}",
                    plugin_name, dependency, required, found
                )
            }
        }
    }
}

impl std::error::Error for PluginError {}

pub trait Plugin: Send + Sync + 'static {
    fn build(&self, app: &mut App);
    fn name(&self) -> &str;

    /// Returns the list of plugins this plugin depends on
    /// Default implementation returns an empty list (no dependencies)
    fn dependencies(&self) -> Vec<PluginDependency> {
        Vec::new()
    }

    /// Returns the version of this plugin
    /// Default implementation returns "0.1.0"
    fn version(&self) -> &str {
        "0.1.0"
    }
}
