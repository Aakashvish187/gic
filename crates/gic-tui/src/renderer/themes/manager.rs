//! # Theme Manager
//!
//! Manages theme selection and lifecycle. Supports switching between
//! built-in themes and is designed for future extension with user-defined
//! themes loaded from TOML files.

use std::collections::HashMap;

use crate::renderer::themes::builtin;
use crate::renderer::themes::theme::Theme;

/// Manages available themes and the currently active theme.
///
/// The theme manager maintains a registry of available themes and
/// provides methods to switch between them. Built-in themes are
/// registered automatically at construction.
///
/// # Future Extension
///
/// - `load_from_file(path)` — Load a user-defined theme from a TOML file.
/// - `register(theme)` — Register a custom theme at runtime.
pub struct ThemeManager {
    /// Registry of available themes by name.
    themes: HashMap<String, Theme>,
    /// Name of the currently active theme.
    active_theme_name: String,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    /// Creates a new theme manager with all built-in themes registered.
    ///
    /// The default active theme is "GIC Dark".
    pub fn new() -> Self {
        let mut themes = HashMap::new();

        let dark = builtin::gic_dark();
        let light = builtin::gic_light();
        let hc = builtin::high_contrast();

        themes.insert(dark.name.clone(), dark);
        themes.insert(light.name.clone(), light);
        themes.insert(hc.name.clone(), hc);

        Self {
            themes,
            active_theme_name: "GIC Dark".to_string(),
        }
    }

    /// Returns a reference to the currently active theme.
    ///
    /// # Panics
    ///
    /// Never panics — falls back to "GIC Dark" if the active theme
    /// name is invalid (should not happen in practice).
    pub fn active_theme(&self) -> &Theme {
        self.themes
            .get(&self.active_theme_name)
            .or_else(|| self.themes.get("GIC Dark"))
            .expect("Built-in GIC Dark theme must always exist")
    }

    /// Returns the name of the currently active theme.
    pub fn active_theme_name(&self) -> &str {
        &self.active_theme_name
    }

    /// Switches to a different theme by name.
    ///
    /// Returns `true` if the theme was found and activated, `false` if
    /// the theme name is unknown (active theme remains unchanged).
    pub fn set_active_theme(&mut self, name: &str) -> bool {
        if self.themes.contains_key(name) {
            self.active_theme_name = name.to_string();
            true
        } else {
            false
        }
    }

    /// Returns a sorted list of available theme names.
    pub fn available_themes(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.themes.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
    }

    /// Returns the total number of registered themes.
    pub fn theme_count(&self) -> usize {
        self.themes.len()
    }

    /// Retrieves a theme by name.
    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }

    /// Registers a custom theme. Overwrites if a theme with the same name exists.
    pub fn register_theme(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), theme);
    }

    /// Cycles to the next theme in alphabetical order.
    ///
    /// Returns the name of the newly active theme.
    pub fn cycle_next(&mut self) -> &str {
        let names = self.available_themes();
        if names.is_empty() {
            return &self.active_theme_name;
        }

        let current_idx = names
            .iter()
            .position(|n| *n == self.active_theme_name)
            .unwrap_or(0);

        let next_idx = (current_idx + 1) % names.len();
        self.active_theme_name = names[next_idx].to_string();
        &self.active_theme_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_creation() {
        let mgr = ThemeManager::new();
        assert_eq!(mgr.active_theme_name(), "GIC Dark");
        assert_eq!(mgr.theme_count(), 3);
    }

    #[test]
    fn test_active_theme() {
        let mgr = ThemeManager::new();
        let theme = mgr.active_theme();
        assert_eq!(theme.name, "GIC Dark");
    }

    #[test]
    fn test_switch_theme() {
        let mut mgr = ThemeManager::new();

        assert!(mgr.set_active_theme("GIC Light"));
        assert_eq!(mgr.active_theme().name, "GIC Light");

        assert!(mgr.set_active_theme("High Contrast"));
        assert_eq!(mgr.active_theme().name, "High Contrast");

        assert!(!mgr.set_active_theme("NonExistent"));
        assert_eq!(mgr.active_theme().name, "High Contrast"); // Unchanged
    }

    #[test]
    fn test_available_themes() {
        let mgr = ThemeManager::new();
        let themes = mgr.available_themes();
        assert_eq!(themes.len(), 3);
        assert!(themes.contains(&"GIC Dark"));
        assert!(themes.contains(&"GIC Light"));
        assert!(themes.contains(&"High Contrast"));
    }

    #[test]
    fn test_get_theme() {
        let mgr = ThemeManager::new();
        assert!(mgr.get_theme("GIC Dark").is_some());
        assert!(mgr.get_theme("NonExistent").is_none());
    }

    #[test]
    fn test_register_custom_theme() {
        let mut mgr = ThemeManager::new();
        let custom = builtin::gic_dark(); // Clone dark as base
        let mut custom_theme = custom;
        custom_theme.name = "My Custom Theme".to_string();

        mgr.register_theme(custom_theme);
        assert_eq!(mgr.theme_count(), 4);
        assert!(mgr.get_theme("My Custom Theme").is_some());
    }

    #[test]
    fn test_cycle_next() {
        let mut mgr = ThemeManager::new();
        let initial = mgr.active_theme_name().to_string();

        let next = mgr.cycle_next().to_string();
        assert_ne!(initial, next);

        // Cycle through all themes
        for _ in 0..3 {
            mgr.cycle_next();
        }
        // Should be back to same position after 3 cycles (3 themes)
    }
}
