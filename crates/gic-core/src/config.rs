use serde::{Deserialize, Serialize};

/// Main application domain configuration structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub app_name: String,
    pub logging: LogConfig,
    pub ui: UIConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: "GIC".to_string(),
            logging: LogConfig::default(),
            ui: UIConfig::default(),
        }
    }
}

/// Configuration settings for structured logging infrastructure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub log_dir: String,
    pub log_file: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            log_dir: "logs".to_string(),
            log_file: "gic.log".to_string(),
        }
    }
}

/// Configuration settings for the Terminal UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: String,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_app_config() {
        let config = AppConfig::default();
        assert_eq!(config.app_name, "GIC");
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.logging.log_dir, "logs");
        assert_eq!(config.logging.log_file, "gic.log");
        assert_eq!(config.ui.theme, "default");
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap_or_default();
        assert!(json.contains("\"app_name\":\"GIC\""));
    }
}
