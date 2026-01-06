use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration file name
const CONFIG_FILE_NAME: &str = ".essentialscode.toml";

/// Global configuration file name (in home directory)
const GLOBAL_CONFIG_FILE_NAME: &str = "essentialscode.toml";

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub scan: ScanConfig,

    #[serde(default)]
    pub languages: LanguagesConfig,

    #[serde(default)]
    pub output: OutputConfig,
}

/// Scanning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Maximum directory depth for scanning (default: 5)
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,

    /// Directories to ignore during scanning
    #[serde(default = "default_ignore")]
    pub ignore: Vec<String>,

    /// Whether to run language-specific linters
    #[serde(default = "default_true")]
    pub run_linters: bool,

    /// Whether to run files to check for runtime errors
    #[serde(default = "default_true")]
    pub run_files: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            max_depth: default_max_depth(),
            ignore: default_ignore(),
            run_linters: true,
            run_files: true,
        }
    }
}

/// Languages configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LanguagesConfig {
    /// Enabled languages (empty = all supported)
    #[serde(default)]
    pub enabled: Vec<String>,

    /// Disabled languages
    #[serde(default)]
    pub disabled: Vec<String>,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Use colors in output
    #[serde(default = "default_true")]
    pub colors: bool,

    /// Show hints in output
    #[serde(default = "default_true")]
    pub show_hints: bool,

    /// Show diffs in fix suggestions
    #[serde(default = "default_true")]
    pub show_diffs: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            colors: true,
            show_hints: true,
            show_diffs: true,
        }
    }
}

fn default_max_depth() -> usize {
    5
}

fn default_ignore() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        ".git".to_string(),
        "target".to_string(),
        ".venv".to_string(),
        "venv".to_string(),
        "__pycache__".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".next".to_string(),
    ]
}

fn default_true() -> bool {
    true
}

#[allow(dead_code)]
impl Config {
    /// Load configuration from project directory or global config
    pub fn load(project_path: Option<&Path>) -> Result<Self> {
        // Try loading from project directory first
        if let Some(path) = project_path {
            let config_path = path.join(CONFIG_FILE_NAME);
            if config_path.exists() {
                return Self::load_from_file(&config_path);
            }
        }

        // Try loading from home directory
        if let Some(home) = dirs::home_dir() {
            let global_config = home.join(".config").join(GLOBAL_CONFIG_FILE_NAME);
            if global_config.exists() {
                return Self::load_from_file(&global_config);
            }
        }

        // Return default config if no file found
        Ok(Self::default())
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the default config file path for a project
    pub fn project_config_path(project_path: &Path) -> PathBuf {
        project_path.join(CONFIG_FILE_NAME)
    }

    /// Get the global config file path
    pub fn global_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".config").join(GLOBAL_CONFIG_FILE_NAME))
    }

    /// Check if a path should be ignored based on config
    pub fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.scan
            .ignore
            .iter()
            .any(|ignore| path_str.contains(ignore))
    }

    /// Check if a language is enabled
    pub fn is_language_enabled(&self, lang: &str) -> bool {
        let lang_lower = lang.to_lowercase();

        // If disabled list contains this language, it's disabled
        if self
            .languages
            .disabled
            .iter()
            .any(|l| l.to_lowercase() == lang_lower)
        {
            return false;
        }

        // If enabled list is empty, all languages are enabled
        if self.languages.enabled.is_empty() {
            return true;
        }

        // Otherwise, check if it's in the enabled list
        self.languages
            .enabled
            .iter()
            .any(|l| l.to_lowercase() == lang_lower)
    }

    /// Generate example configuration content
    pub fn example_config() -> String {
        r#"# EssentialsCode Configuration
# Place this file in your project root as .essentialscode.toml
# or in ~/.config/essentialscode.toml for global settings

[scan]
# Maximum directory depth for scanning
max_depth = 5

# Directories to ignore during scanning
ignore = [
    "node_modules",
    ".git",
    "target",
    ".venv",
    "venv",
    "__pycache__",
    "dist",
    "build",
    ".next",
]

# Run language-specific linters (e.g., pylint for Python)
run_linters = true

# Run files to detect runtime errors
run_files = true

[languages]
# Languages to check (empty = all supported)
# enabled = ["python", "rust", "typescript"]

# Languages to skip
# disabled = ["cpp"]

[output]
# Use colors in terminal output
colors = true

# Show hints for fixing errors
show_hints = true

# Show before/after diffs in fix suggestions
show_diffs = true
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.scan.max_depth, 5);
        assert!(config.scan.ignore.contains(&"node_modules".to_string()));
        assert!(config.scan.run_linters);
        assert!(config.output.colors);
    }

    #[test]
    fn test_should_ignore() {
        let config = Config::default();
        assert!(config.should_ignore(Path::new("/project/node_modules/package")));
        assert!(config.should_ignore(Path::new("/project/.git/config")));
        assert!(!config.should_ignore(Path::new("/project/src/main.rs")));
    }

    #[test]
    fn test_is_language_enabled_default() {
        let config = Config::default();
        assert!(config.is_language_enabled("python"));
        assert!(config.is_language_enabled("rust"));
        assert!(config.is_language_enabled("javascript"));
    }

    #[test]
    fn test_is_language_enabled_with_list() {
        let mut config = Config::default();
        config.languages.enabled = vec!["python".to_string(), "rust".to_string()];

        assert!(config.is_language_enabled("python"));
        assert!(config.is_language_enabled("rust"));
        assert!(!config.is_language_enabled("javascript"));
    }

    #[test]
    fn test_is_language_disabled() {
        let mut config = Config::default();
        config.languages.disabled = vec!["cpp".to_string()];

        assert!(config.is_language_enabled("python"));
        assert!(!config.is_language_enabled("cpp"));
    }

    #[test]
    fn test_load_from_file() {
        let temp_dir = std::env::temp_dir().join("ess_config_test");
        let _ = std::fs::create_dir_all(&temp_dir);

        let config_path = temp_dir.join(".essentialscode.toml");
        let mut file = std::fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"
[scan]
max_depth = 10

[languages]
enabled = ["python"]
"#
        )
        .unwrap();

        let config = Config::load_from_file(&config_path).unwrap();
        assert_eq!(config.scan.max_depth, 10);
        assert!(config.is_language_enabled("python"));
        assert!(!config.is_language_enabled("rust"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_save_to_file() {
        let temp_dir = std::env::temp_dir().join("ess_config_save_test");
        let _ = std::fs::create_dir_all(&temp_dir);

        let config_path = temp_dir.join("test_config.toml");
        let config = Config::default();
        config.save_to_file(&config_path).unwrap();

        assert!(config_path.exists());
        let loaded = Config::load_from_file(&config_path).unwrap();
        assert_eq!(loaded.scan.max_depth, config.scan.max_depth);

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_example_config_is_valid_toml() {
        let example = Config::example_config();
        let result: Result<Config, _> = toml::from_str(&example);
        assert!(result.is_ok());
    }

    #[test]
    fn test_case_insensitive_language_check() {
        let mut config = Config::default();
        config.languages.enabled = vec!["Python".to_string()];

        assert!(config.is_language_enabled("python"));
        assert!(config.is_language_enabled("Python"));
        assert!(config.is_language_enabled("PYTHON"));
    }
}
