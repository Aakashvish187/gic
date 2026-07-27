//! Theme system tests.

use crate::renderer::syntax::token::TokenKind;
use crate::renderer::themes::{builtin, ThemeManager};

#[test]
fn test_all_builtin_themes_have_names() {
    assert!(!builtin::gic_dark().name.is_empty());
    assert!(!builtin::gic_light().name.is_empty());
    assert!(!builtin::high_contrast().name.is_empty());
}

#[test]
fn test_all_token_kinds_have_styles() {
    let theme = builtin::gic_dark();
    let kinds = [
        TokenKind::Keyword,
        TokenKind::String,
        TokenKind::Number,
        TokenKind::Comment,
        TokenKind::Operator,
        TokenKind::Type,
        TokenKind::Function,
        TokenKind::Constant,
        TokenKind::Attribute,
        TokenKind::Variable,
        TokenKind::Error,
        TokenKind::Punctuation,
        TokenKind::PlainText,
    ];

    for kind in &kinds {
        let style = theme.style_for_token(*kind);
        // Every token kind should produce a style with a foreground color
        assert!(
            style.fg.is_some(),
            "Token {:?} has no foreground color",
            kind
        );
    }
}

#[test]
fn test_theme_manager_cycle_returns_to_start() {
    let mut mgr = ThemeManager::new();
    let start = mgr.active_theme_name().to_string();

    let count = mgr.theme_count();
    for _ in 0..count {
        mgr.cycle_next();
    }

    assert_eq!(mgr.active_theme_name(), start);
}

#[test]
fn test_theme_manager_all_themes_accessible() {
    let mgr = ThemeManager::new();
    for name in mgr.available_themes() {
        assert!(
            mgr.get_theme(name).is_some(),
            "Theme '{}' listed but not accessible",
            name
        );
    }
}

#[test]
fn test_themes_have_distinct_backgrounds() {
    let dark = builtin::gic_dark();
    let light = builtin::gic_light();
    let hc = builtin::high_contrast();

    assert_ne!(dark.background, light.background);
    assert_ne!(dark.background, hc.background);
    assert_ne!(light.background, hc.background);
}
