use crossterm::{
    cursor::{Hide, SetCursorStyle, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gic_core::GicError;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};

/// Production-grade Terminal Engine manager implementing RAII lifecycle safety.
pub struct TerminalEngine {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    mouse_captured: bool,
}

impl TerminalEngine {
    /// Initializes terminal in full screen alternate screen mode with mouse capture enabled.
    pub fn new(enable_mouse: bool) -> Result<Self, GicError> {
        enable_raw_mode().map_err(|e| {
            GicError::Terminal(format!("Failed to enable terminal raw mode: {}", e))
        })?;

        let mut stdout = io::stdout();
        if let Err(e) = execute!(stdout, EnterAlternateScreen, SetCursorStyle::SteadyBar) {
            let _ = disable_raw_mode();
            return Err(GicError::Terminal(format!(
                "Failed to enter alternate screen: {}",
                e
            )));
        }

        let mut mouse_captured = false;
        if enable_mouse {
            if let Err(e) = execute!(stdout, EnableMouseCapture) {
                tracing::warn!("Failed to enable mouse capture: {}", e);
            } else {
                mouse_captured = true;
            }
        }

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).map_err(|e| {
            GicError::Terminal(format!(
                "Failed to initialize Ratatui terminal backend: {}",
                e
            ))
        })?;

        Ok(Self {
            terminal,
            mouse_captured,
        })
    }

    /// Dynamically toggles mouse event capture on/off.
    pub fn set_mouse_capture(&mut self, enable: bool) -> Result<(), GicError> {
        if enable && !self.mouse_captured {
            execute!(self.terminal.backend_mut(), EnableMouseCapture).map_err(|e| {
                GicError::Terminal(format!("Failed to enable mouse capture: {}", e))
            })?;
            self.mouse_captured = true;
        } else if !enable && self.mouse_captured {
            execute!(self.terminal.backend_mut(), DisableMouseCapture).map_err(|e| {
                GicError::Terminal(format!("Failed to disable mouse capture: {}", e))
            })?;
            self.mouse_captured = false;
        }
        Ok(())
    }

    /// Queries current terminal screen dimensions (width, height).
    pub fn size(&self) -> Result<(u16, u16), GicError> {
        let size = self
            .terminal
            .size()
            .map_err(|e| GicError::Terminal(e.to_string()))?;
        Ok((size.width, size.height))
    }

    /// Grants mutable access to the internal Ratatui terminal.
    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalEngine {
    fn drop(&mut self) {
        if self.mouse_captured {
            let _ = execute!(self.terminal.backend_mut(), DisableMouseCapture);
        }
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            SetCursorStyle::DefaultUserShape
        );
        let _ = disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_engine_drop_safety() {
        assert!(std::mem::needs_drop::<TerminalEngine>());
    }
}
