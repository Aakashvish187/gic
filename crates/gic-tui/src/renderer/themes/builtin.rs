//! # Built-in Themes
//!
//! Three production-quality themes shipped with GIC:
//!
//! - **GIC Dark** — The default theme. A modern dark color scheme inspired
//!   by popular IDE themes (One Dark, Catppuccin, Tokyo Night).
//! - **GIC Light** — A clean light theme for bright environments.
//! - **High Contrast** — Maximum readability for accessibility.

use ratatui::style::Color;

use crate::renderer::themes::theme::{SyntaxColors, Theme};

/// Creates the default GIC Dark theme.
///
/// Color palette inspired by modern dark IDE themes with carefully
/// chosen hue/saturation/lightness values for readability and aesthetics.
pub fn gic_dark() -> Theme {
    Theme {
        name: "GIC Dark".to_string(),

        background: Color::Rgb(30, 30, 46),
        foreground: Color::Rgb(205, 214, 244),

        cursor: Color::Rgb(245, 224, 220),
        cursor_line: Color::Rgb(45, 45, 65),

        selection: Color::Rgb(68, 71, 110),

        line_number: Color::Rgb(88, 91, 112),
        line_number_active: Color::Rgb(205, 214, 244),
        gutter_bg: Color::Rgb(30, 30, 46),

        status_bar_bg: Color::Rgb(24, 24, 37),
        status_bar_fg: Color::Rgb(166, 173, 200),
        status_bar_accent: Color::Rgb(137, 180, 250),
        status_bar_secondary: Color::Rgb(88, 91, 112),

        syntax: SyntaxColors {
            keyword: Color::Rgb(203, 166, 247),     // Mauve / Purple
            string: Color::Rgb(166, 227, 161),      // Green
            number: Color::Rgb(250, 179, 135),      // Peach / Orange
            comment: Color::Rgb(108, 112, 134),     // Gray (dimmed)
            operator: Color::Rgb(148, 226, 213),    // Teal
            type_name: Color::Rgb(249, 226, 175),   // Yellow
            function: Color::Rgb(137, 180, 250),    // Blue
            constant: Color::Rgb(250, 179, 135),    // Peach
            attribute: Color::Rgb(245, 194, 231),   // Pink
            variable: Color::Rgb(205, 214, 244),    // Text
            error: Color::Rgb(243, 139, 168),       // Red
            punctuation: Color::Rgb(147, 153, 178), // Overlay
        },
    }
}

/// Creates the GIC Light theme.
///
/// Designed for bright environments with high contrast text on a
/// light background.
pub fn gic_light() -> Theme {
    Theme {
        name: "GIC Light".to_string(),

        background: Color::Rgb(239, 241, 245),
        foreground: Color::Rgb(76, 79, 105),

        cursor: Color::Rgb(30, 30, 46),
        cursor_line: Color::Rgb(220, 224, 232),

        selection: Color::Rgb(172, 176, 190),

        line_number: Color::Rgb(140, 143, 161),
        line_number_active: Color::Rgb(76, 79, 105),
        gutter_bg: Color::Rgb(239, 241, 245),

        status_bar_bg: Color::Rgb(204, 208, 218),
        status_bar_fg: Color::Rgb(76, 79, 105),
        status_bar_accent: Color::Rgb(30, 102, 245),
        status_bar_secondary: Color::Rgb(140, 143, 161),

        syntax: SyntaxColors {
            keyword: Color::Rgb(136, 57, 239),      // Purple
            string: Color::Rgb(64, 160, 43),        // Green
            number: Color::Rgb(254, 100, 11),       // Orange
            comment: Color::Rgb(140, 143, 161),     // Gray
            operator: Color::Rgb(23, 146, 153),     // Teal
            type_name: Color::Rgb(223, 142, 29),    // Yellow
            function: Color::Rgb(30, 102, 245),     // Blue
            constant: Color::Rgb(254, 100, 11),     // Orange
            attribute: Color::Rgb(234, 118, 203),   // Pink
            variable: Color::Rgb(76, 79, 105),      // Text
            error: Color::Rgb(210, 15, 57),         // Red
            punctuation: Color::Rgb(108, 111, 133), // Subtext
        },
    }
}

/// Creates the High Contrast theme.
///
/// Maximizes color contrast for accessibility. Uses pure black/white
/// backgrounds with saturated, distinct syntax colors.
pub fn high_contrast() -> Theme {
    Theme {
        name: "High Contrast".to_string(),

        background: Color::Rgb(0, 0, 0),
        foreground: Color::Rgb(255, 255, 255),

        cursor: Color::Rgb(255, 255, 255),
        cursor_line: Color::Rgb(30, 30, 30),

        selection: Color::Rgb(0, 80, 160),

        line_number: Color::Rgb(150, 150, 150),
        line_number_active: Color::Rgb(255, 255, 255),
        gutter_bg: Color::Rgb(0, 0, 0),

        status_bar_bg: Color::Rgb(40, 40, 40),
        status_bar_fg: Color::Rgb(255, 255, 255),
        status_bar_accent: Color::Rgb(0, 150, 255),
        status_bar_secondary: Color::Rgb(180, 180, 180),

        syntax: SyntaxColors {
            keyword: Color::Rgb(255, 100, 255),     // Bright Magenta
            string: Color::Rgb(100, 255, 100),      // Bright Green
            number: Color::Rgb(255, 180, 80),       // Bright Orange
            comment: Color::Rgb(128, 128, 128),     // Gray
            operator: Color::Rgb(0, 255, 255),      // Cyan
            type_name: Color::Rgb(255, 255, 100),   // Bright Yellow
            function: Color::Rgb(100, 180, 255),    // Bright Blue
            constant: Color::Rgb(255, 160, 100),    // Orange
            attribute: Color::Rgb(255, 150, 200),   // Pink
            variable: Color::Rgb(255, 255, 255),    // White
            error: Color::Rgb(255, 50, 50),         // Bright Red
            punctuation: Color::Rgb(200, 200, 200), // Light Gray
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gic_dark_theme() {
        let theme = gic_dark();
        assert_eq!(theme.name, "GIC Dark");
        assert_ne!(theme.background, theme.foreground);
        assert_ne!(theme.cursor, theme.background);
    }

    #[test]
    fn test_gic_light_theme() {
        let theme = gic_light();
        assert_eq!(theme.name, "GIC Light");
        assert_ne!(theme.background, theme.foreground);
    }

    #[test]
    fn test_high_contrast_theme() {
        let theme = high_contrast();
        assert_eq!(theme.name, "High Contrast");
        assert_eq!(theme.background, Color::Rgb(0, 0, 0));
        assert_eq!(theme.foreground, Color::Rgb(255, 255, 255));
    }

    #[test]
    fn test_all_themes_have_distinct_syntax_colors() {
        for theme_fn in [gic_dark, gic_light, high_contrast] {
            let theme = theme_fn();
            // Keywords and strings should be different colors
            assert_ne!(
                theme.syntax.keyword, theme.syntax.string,
                "Theme {} has same keyword and string colors",
                theme.name
            );
            // Comments should be different from keywords
            assert_ne!(
                theme.syntax.comment, theme.syntax.keyword,
                "Theme {} has same comment and keyword colors",
                theme.name
            );
        }
    }

    #[test]
    fn test_themes_are_clonable() {
        let theme = gic_dark();
        let cloned = theme.clone();
        assert_eq!(theme, cloned);
    }
}
