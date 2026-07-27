//! # Syntax Renderer
//!
//! Converts [`HighlightedToken`] sequences into themed ratatui [`Span`]
//! sequences. This is the bridge between the syntax engine and the
//! rendering pipeline.

use ratatui::style::Style;
use ratatui::text::Span;

use crate::renderer::syntax::token::HighlightedToken;
use crate::renderer::themes::Theme;

/// Converts syntax tokens to themed ratatui spans.
///
/// The syntax renderer is stateless — it maps each token's `TokenKind`
/// to the appropriate color from the active theme.
pub struct SyntaxRenderer;

impl SyntaxRenderer {
    /// Converts a sequence of highlighted tokens to ratatui spans.
    ///
    /// Each token is styled according to its `TokenKind` using colors
    /// from the provided theme.
    pub fn tokens_to_spans<'a>(tokens: &'a [HighlightedToken], theme: &Theme) -> Vec<Span<'a>> {
        tokens
            .iter()
            .map(|token| {
                let style = theme.style_for_token(token.kind);
                Span::styled(token.text.clone(), style)
            })
            .collect()
    }

    /// Converts tokens to spans with a custom background color override.
    ///
    /// Used for current-line highlighting where the background must be
    /// different from the default theme background.
    pub fn tokens_to_spans_with_bg<'a>(
        tokens: &'a [HighlightedToken],
        theme: &Theme,
        bg: ratatui::style::Color,
    ) -> Vec<Span<'a>> {
        tokens
            .iter()
            .map(|token| {
                let base_style = theme.style_for_token(token.kind);
                let style = Style::default()
                    .fg(base_style.fg.unwrap_or(theme.foreground))
                    .bg(bg)
                    .add_modifier(base_style.add_modifier);
                Span::styled(token.text.clone(), style)
            })
            .collect()
    }

    /// Returns a single plain-text span for lines without syntax highlighting.
    pub fn plain_span(text: &str, theme: &Theme) -> Span<'static> {
        Span::styled(
            text.to_string(),
            Style::default().fg(theme.foreground).bg(theme.background),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::syntax::token::TokenKind;
    use crate::renderer::themes::builtin;

    #[test]
    fn test_tokens_to_spans() {
        let theme = builtin::gic_dark();
        let tokens = vec![
            HighlightedToken::new(TokenKind::Keyword, 0, 2, "fn".to_string()),
            HighlightedToken::new(TokenKind::PlainText, 2, 3, " ".to_string()),
            HighlightedToken::new(TokenKind::Function, 3, 7, "main".to_string()),
        ];

        let spans = SyntaxRenderer::tokens_to_spans(&tokens, &theme);
        assert_eq!(spans.len(), 3);
    }

    #[test]
    fn test_tokens_to_spans_with_bg() {
        let theme = builtin::gic_dark();
        let tokens = vec![HighlightedToken::new(
            TokenKind::Keyword,
            0,
            3,
            "let".to_string(),
        )];

        let spans = SyntaxRenderer::tokens_to_spans_with_bg(
            &tokens,
            &theme,
            ratatui::style::Color::Rgb(40, 40, 60),
        );
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_plain_span() {
        let theme = builtin::gic_dark();
        let span = SyntaxRenderer::plain_span("hello", &theme);
        assert_eq!(span.content.as_ref(), "hello");
    }

    #[test]
    fn test_empty_tokens() {
        let theme = builtin::gic_dark();
        let spans = SyntaxRenderer::tokens_to_spans(&[], &theme);
        assert!(spans.is_empty());
    }
}
