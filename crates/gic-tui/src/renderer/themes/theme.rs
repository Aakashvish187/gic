//! # Theme Definition
//!
//! The core `Theme` struct defining all color slots used by the rendering
//! engine. Every visual element in the editor references a theme color,
//! ensuring consistent and customizable appearance.

use ratatui::style::{Color, Modifier, Style};

use crate::renderer::syntax::token::TokenKind;

/// Colors for syntax-highlighted tokens.
///
/// Each field corresponds to a [`TokenKind`] variant. The syntax renderer
/// uses this struct to resolve token kinds to visual styles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxColors {
    /// Color for language keywords.
    pub keyword: Color,
    /// Color for string literals.
    pub string: Color,
    /// Color for numeric literals.
    pub number: Color,
    /// Color for comments.
    pub comment: Color,
    /// Color for operators.
    pub operator: Color,
    /// Color for type names.
    pub type_name: Color,
    /// Color for function names.
    pub function: Color,
    /// Color for constants.
    pub constant: Color,
    /// Color for attributes/decorators.
    pub attribute: Color,
    /// Color for variables.
    pub variable: Color,
    /// Color for error tokens.
    pub error: Color,
    /// Color for punctuation.
    pub punctuation: Color,
}

/// Complete theme definition for the GIC editor.
///
/// Themes are value types — they can be cloned, compared, and stored.
/// The theme defines colors for every visual element in the editor,
/// providing a single source of truth for the UI appearance.
///
/// # Future Extension
///
/// User-defined themes will be loaded from TOML files and converted
/// into `Theme` instances. The struct is designed to be serializable
/// with serde in a future milestone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    /// Human-readable theme name.
    pub name: String,

    // ─── Editor Background ───────────────────────────────────────
    /// Main editor background color.
    pub background: Color,
    /// Main editor foreground (text) color.
    pub foreground: Color,

    // ─── Cursor ──────────────────────────────────────────────────
    /// Cursor color.
    pub cursor: Color,
    /// Current line highlight background.
    pub cursor_line: Color,

    // ─── Selection ───────────────────────────────────────────────
    /// Selection highlight background.
    pub selection: Color,

    // ─── Line Numbers ────────────────────────────────────────────
    /// Line number text color (non-active lines).
    pub line_number: Color,
    /// Active (current) line number text color.
    pub line_number_active: Color,
    /// Line number gutter background.
    pub gutter_bg: Color,

    // ─── Status Bar ──────────────────────────────────────────────
    /// Status bar background color.
    pub status_bar_bg: Color,
    /// Status bar foreground (text) color.
    pub status_bar_fg: Color,
    /// Status bar accent color (for mode indicators, etc.).
    pub status_bar_accent: Color,
    /// Status bar secondary info color.
    pub status_bar_secondary: Color,

    // ─── Syntax ──────────────────────────────────────────────────
    /// Syntax highlighting colors.
    pub syntax: SyntaxColors,

    // ─── Panels ──────────────────────────────────────────────────
    /// Panel background color (explorer, intelligence, bottom).
    pub panel_bg: Color,
    /// Panel border/separator color.
    pub panel_border: Color,
    /// Panel header text color.
    pub panel_header: Color,
    /// Top bar background.
    pub top_bar_bg: Color,
    /// Top bar foreground.
    pub top_bar_fg: Color,
    /// Top bar accent (brand color).
    pub top_bar_accent: Color,
    /// File explorer active item background.
    pub explorer_active: Color,
    /// Diagnostic error color.
    pub diagnostic_error: Color,
    /// Diagnostic warning color.
    pub diagnostic_warning: Color,
    /// Diagnostic info/hint color.
    pub diagnostic_info: Color,
}

impl Theme {
    /// Returns a ratatui `Style` for the given token kind.
    ///
    /// This is the primary method used by the syntax renderer to convert
    /// token categories to visual styles.
    pub fn style_for_token(&self, kind: TokenKind) -> Style {
        let (color, modifier) = match kind {
            TokenKind::Keyword => (self.syntax.keyword, Modifier::BOLD),
            TokenKind::String => (self.syntax.string, Modifier::empty()),
            TokenKind::Number => (self.syntax.number, Modifier::empty()),
            TokenKind::Comment => (self.syntax.comment, Modifier::ITALIC),
            TokenKind::Operator => (self.syntax.operator, Modifier::empty()),
            TokenKind::Type => (self.syntax.type_name, Modifier::empty()),
            TokenKind::Function => (self.syntax.function, Modifier::empty()),
            TokenKind::Constant => (self.syntax.constant, Modifier::BOLD),
            TokenKind::Attribute => (self.syntax.attribute, Modifier::empty()),
            TokenKind::Variable => (self.syntax.variable, Modifier::empty()),
            TokenKind::Error => (self.syntax.error, Modifier::UNDERLINED),
            TokenKind::Punctuation => (self.syntax.punctuation, Modifier::empty()),
            TokenKind::PlainText => (self.foreground, Modifier::empty()),
        };

        Style::default().fg(color).add_modifier(modifier)
    }

    /// Returns the style for the status bar.
    pub fn status_bar_style(&self) -> Style {
        Style::default()
            .fg(self.status_bar_fg)
            .bg(self.status_bar_bg)
    }

    /// Returns the style for the status bar mode indicator.
    pub fn status_bar_mode_style(&self) -> Style {
        Style::default()
            .fg(self.status_bar_bg)
            .bg(self.status_bar_accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Returns the style for line numbers.
    pub fn line_number_style(&self) -> Style {
        Style::default().fg(self.line_number).bg(self.gutter_bg)
    }

    /// Returns the style for the active (current) line number.
    pub fn active_line_number_style(&self) -> Style {
        Style::default()
            .fg(self.line_number_active)
            .bg(self.gutter_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Returns the style for the current line highlight.
    pub fn current_line_style(&self) -> Style {
        Style::default().bg(self.cursor_line)
    }

    /// Returns the style for selection highlighting.
    pub fn selection_style(&self) -> Style {
        Style::default().bg(self.selection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::themes::builtin;

    #[test]
    fn test_theme_style_for_token() {
        let theme = builtin::gic_dark();

        let kw_style = theme.style_for_token(TokenKind::Keyword);
        assert_eq!(kw_style.fg, Some(theme.syntax.keyword));

        let comment_style = theme.style_for_token(TokenKind::Comment);
        assert_eq!(comment_style.fg, Some(theme.syntax.comment));
    }

    #[test]
    fn test_theme_status_bar_style() {
        let theme = builtin::gic_dark();
        let style = theme.status_bar_style();
        assert_eq!(style.fg, Some(theme.status_bar_fg));
        assert_eq!(style.bg, Some(theme.status_bar_bg));
    }

    #[test]
    fn test_theme_line_number_styles() {
        let theme = builtin::gic_dark();

        let normal = theme.line_number_style();
        assert_eq!(normal.fg, Some(theme.line_number));

        let active = theme.active_line_number_style();
        assert_eq!(active.fg, Some(theme.line_number_active));
    }

    #[test]
    fn test_theme_clone_eq() {
        let theme1 = builtin::gic_dark();
        let theme2 = theme1.clone();
        assert_eq!(theme1, theme2);
    }
}
