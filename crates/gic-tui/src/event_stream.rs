use crossterm::event::{
    self, Event as CrosstermEvent, KeyCode as CrosstermKeyCode, KeyModifiers as CrosstermModifiers,
    MouseButton as CrosstermMouseButton, MouseEventKind,
};
use gic_core::{
    GicError, InputEvent, KeyCode, KeyInput, KeyModifiers, MouseAction, MouseButton, MouseInput,
    UIConfig,
};
use std::time::Duration;

/// Event listener component converting raw terminal I/O events to domain events.
pub struct EventStream {
    tick_rate: Duration,
}

impl EventStream {
    pub fn new(ui_config: &UIConfig) -> Self {
        Self {
            tick_rate: Duration::from_millis(ui_config.tick_rate_ms),
        }
    }

    /// Polls for next input event or produces a `Tick` event if duration elapses.
    pub fn next_event(&self) -> Result<InputEvent, GicError> {
        if event::poll(self.tick_rate)
            .map_err(|e| GicError::Terminal(format!("Event poll error: {}", e)))?
        {
            match event::read()
                .map_err(|e| GicError::Terminal(format!("Event read error: {}", e)))?
            {
                CrosstermEvent::Key(key_event) => {
                    let modifiers = parse_modifiers(key_event.modifiers);
                    let code = parse_key_code(key_event.code);
                    Ok(InputEvent::Key(KeyInput::new(code, modifiers)))
                }
                CrosstermEvent::Mouse(mouse_event) => {
                    let modifiers = parse_modifiers(mouse_event.modifiers);
                    if let Some(action) = parse_mouse_action(mouse_event.kind) {
                        Ok(InputEvent::Mouse(MouseInput {
                            action,
                            column: mouse_event.column,
                            row: mouse_event.row,
                            modifiers,
                        }))
                    } else {
                        Ok(InputEvent::Tick)
                    }
                }
                CrosstermEvent::Resize(width, height) => Ok(InputEvent::Resize { width, height }),
                CrosstermEvent::Paste(s) => Ok(InputEvent::Paste(s)),
                _ => Ok(InputEvent::Tick),
            }
        } else {
            Ok(InputEvent::Tick)
        }
    }
}

fn parse_modifiers(m: CrosstermModifiers) -> KeyModifiers {
    KeyModifiers {
        shift: m.contains(CrosstermModifiers::SHIFT),
        control: m.contains(CrosstermModifiers::CONTROL),
        alt: m.contains(CrosstermModifiers::ALT),
    }
}

fn parse_key_code(code: CrosstermKeyCode) -> KeyCode {
    match code {
        CrosstermKeyCode::Char(c) => KeyCode::Char(c),
        CrosstermKeyCode::F(num) => KeyCode::F(num),
        CrosstermKeyCode::Backspace => KeyCode::Backspace,
        CrosstermKeyCode::Enter => KeyCode::Enter,
        CrosstermKeyCode::Left => KeyCode::Left,
        CrosstermKeyCode::Right => KeyCode::Right,
        CrosstermKeyCode::Up => KeyCode::Up,
        CrosstermKeyCode::Down => KeyCode::Down,
        CrosstermKeyCode::Home => KeyCode::Home,
        CrosstermKeyCode::End => KeyCode::End,
        CrosstermKeyCode::PageUp => KeyCode::PageUp,
        CrosstermKeyCode::PageDown => KeyCode::PageDown,
        CrosstermKeyCode::Tab => KeyCode::Tab,
        CrosstermKeyCode::BackTab => KeyCode::BackTab,
        CrosstermKeyCode::Delete => KeyCode::Delete,
        CrosstermKeyCode::Esc => KeyCode::Esc,
        _ => KeyCode::Null,
    }
}

fn parse_mouse_action(kind: MouseEventKind) -> Option<MouseAction> {
    match kind {
        MouseEventKind::Down(btn) => parse_mouse_button(btn).map(MouseAction::Press),
        MouseEventKind::Up(btn) => parse_mouse_button(btn).map(MouseAction::Release),
        MouseEventKind::Drag(btn) => parse_mouse_button(btn).map(MouseAction::Drag),
        MouseEventKind::Moved => Some(MouseAction::Moved),
        MouseEventKind::ScrollUp => Some(MouseAction::ScrollUp),
        MouseEventKind::ScrollDown => Some(MouseAction::ScrollDown),
        _ => None,
    }
}

fn parse_mouse_button(btn: CrosstermMouseButton) -> Option<MouseButton> {
    match btn {
        CrosstermMouseButton::Left => Some(MouseButton::Left),
        CrosstermMouseButton::Right => Some(MouseButton::Right),
        CrosstermMouseButton::Middle => Some(MouseButton::Middle),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modifiers() {
        let m = CrosstermModifiers::CONTROL | CrosstermModifiers::SHIFT;
        let parsed = parse_modifiers(m);
        assert!(parsed.control);
        assert!(parsed.shift);
        assert!(!parsed.alt);
    }

    #[test]
    fn test_parse_key_code() {
        assert_eq!(
            parse_key_code(CrosstermKeyCode::Char('a')),
            KeyCode::Char('a')
        );
        assert_eq!(parse_key_code(CrosstermKeyCode::Enter), KeyCode::Enter);
        assert_eq!(parse_key_code(CrosstermKeyCode::Esc), KeyCode::Esc);
    }

    #[test]
    fn test_parse_mouse_action() {
        assert_eq!(
            parse_mouse_action(MouseEventKind::Down(CrosstermMouseButton::Left)),
            Some(MouseAction::Press(MouseButton::Left))
        );
        assert_eq!(
            parse_mouse_action(MouseEventKind::ScrollUp),
            Some(MouseAction::ScrollUp)
        );
    }
}
