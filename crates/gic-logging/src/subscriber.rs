use gic_core::{GicError, LogConfig};
use std::fs;
use std::path::Path;
use tracing_subscriber::EnvFilter;

/// Initializes structured logging using `tracing-subscriber`.
///
/// Ensures log directory exists and hooks into std panic handler to write
/// structured log events safely without garbling terminal displays.
pub fn init_logging(config: &LogConfig) -> Result<(), GicError> {
    let log_dir = Path::new(&config.log_dir);
    if !log_dir.exists() {
        fs::create_dir_all(log_dir).map_err(|e| {
            GicError::Logging(format!(
                "Failed to create log directory '{:?}': {}",
                log_dir, e
            ))
        })?;
    }

    let file_path = log_dir.join(&config.log_file);
    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .map_err(|e| {
            GicError::Logging(format!("Failed to open log file '{:?}': {}", file_path, e))
        })?;

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(log_file)
        .with_ansi(false)
        .try_init()
        .map_err(|e| {
            GicError::Logging(format!("Failed to set global default subscriber: {}", e))
        })?;

    setup_panic_hook();

    tracing::info!(
        log_file = %file_path.display(),
        level = %config.level,
        "Logging system initialized successfully"
    );

    Ok(())
}

/// Registers a panic hook that logs panics before propagating them.
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        tracing::error!(panic_details = %panic_info, "Unhandled application panic occurred");
        original_hook(panic_info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_config_path_creation() {
        let temp_dir = std::env::temp_dir().join("gic_logging_test");
        let config = LogConfig {
            level: "debug".into(),
            log_dir: temp_dir.to_string_lossy().to_string(),
            log_file: "test_output.log".into(),
        };

        // Cleanup before test
        let _ = fs::remove_dir_all(&temp_dir);

        // Verify init_logging creates missing directories
        let _result = init_logging(&config);
        // Note: try_init might return error if subscriber already initialized in test runner,
        // but directory creation must succeed.
        assert!(temp_dir.exists());

        // Cleanup after test
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
