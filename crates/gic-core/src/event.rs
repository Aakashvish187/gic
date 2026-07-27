use std::fmt;

/// Modifiers for keyboard and mouse inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
}

/// Domain keyboard inputs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Char(char),
    F(u8),
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Esc,
    Null,
}

/// Keyboard event input payload.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyInput {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyInput {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn char(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::default(),
        }
    }
}

/// Mouse action types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseAction {
    Press(MouseButton),
    Release(MouseButton),
    Drag(MouseButton),
    Moved,
    ScrollUp,
    ScrollDown,
}

/// Mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Mouse event payload.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MouseInput {
    pub action: MouseAction,
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

/// Central domain input event enum for GIC engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Key(KeyInput),
    Mouse(MouseInput),
    Resize { width: u16, height: u16 },
    Tick,
    Paste(String),
}

impl fmt::Display for InputEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(k) => write!(f, "Key({:?})", k.code),
            Self::Mouse(m) => write!(f, "Mouse({:?} at {},{})", m.action, m.column, m.row),
            Self::Resize { width, height } => write!(f, "Resize({}x{})", width, height),
            Self::Tick => write!(f, "Tick"),
            Self::Paste(s) => write!(f, "Paste({} bytes)", s.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_input_creation() {
        let key = KeyInput::char('q');
        assert_eq!(key.code, KeyCode::Char('q'));
        assert_eq!(key.modifiers, KeyModifiers::default());
    }

    #[test]
    fn test_mouse_input_creation() {
        let mouse = MouseInput {
            action: MouseAction::Press(MouseButton::Left),
            column: 10,
            row: 20,
            modifiers: KeyModifiers::default(),
        };
        assert_eq!(mouse.column, 10);
        assert_eq!(mouse.row, 20);
    }

    #[test]
    fn test_input_event_display() {
        let event = InputEvent::Resize {
            width: 120,
            height: 40,
        };
        assert_eq!(format!("{}", event), "Resize(120x40)");

        let tick = InputEvent::Tick;
        assert_eq!(format!("{}", tick), "Tick");
    }
}
