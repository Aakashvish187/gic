use gic_core::{AppConfig, GicError};
use std::fs;
use std::path::Path;

/// Configuration loader utility responsible for reading, parsing, and validating TOML config files.
pub struct ConfigLoader;

impl ConfigLoader {
    /// Loads application configuration from a given file path.
    /// If the file does not exist, returns the default configuration.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<AppConfig, GicError> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Ok(AppConfig::default());
        }

        let content = fs::read_to_string(path_ref).map_err(|e| {
            GicError::Config(format!(
                "Failed to read config file '{:?}': {}",
                path_ref, e
            ))
        })?;

        Self::parse_toml(&content)
    }

    /// Parses TOML string content into `AppConfig`.
    pub fn parse_toml(content: &str) -> Result<AppConfig, GicError> {
        toml::from_str(content)
            .map_err(|e| GicError::Config(format!("Failed to parse TOML configuration: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_non_existent_file_returns_default() {
        let path = Path::new("non_existent_config_file_12345.toml");
        let config = ConfigLoader::load_from_file(path).expect("Should return default config");
        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn test_parse_valid_toml() {
        let toml_str = r#"
            app_name = "GIC Test"
            [logging]
            level = "debug"
            log_dir = "test_logs"
            log_file = "test.log"
            [ui]
            theme = "custom"
        "#;

        let config = ConfigLoader::parse_toml(toml_str).expect("Valid TOML should parse");
        assert_eq!(config.app_name, "GIC Test");
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.ui.theme, "custom");
    }

    #[test]
    fn test_parse_invalid_toml_returns_error() {
        let invalid_toml = r#"
            app_name = GIC Test (missing quotes)
        "#;

        let result = ConfigLoader::parse_toml(invalid_toml);
        assert!(result.is_err());
        if let Err(GicError::Config(msg)) = result {
            assert!(msg.contains("Failed to parse TOML"));
        } else {
            panic!("Expected GicError::Config");
        }
    }
}
