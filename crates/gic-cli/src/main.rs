mod cli;

use anyhow::Result;
use cli::CliOptions;
use gic_config::ConfigLoader;
use gic_core::{
    AboutProvider, DefaultAboutProvider, EngineState, InputEvent, KeyCode, MouseAction,
    ShutdownReason,
};
use gic_logging::init_logging;
use gic_tui::{EventStream, RenderEngine, StatusBar, TerminalEngine};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};
use tracing::info;

fn main() -> Result<()> {
    // 1. Parse Command Line Options
    let options = CliOptions::parse();

    // Check for About or Version flags before bootstrapping TUI
    if options.about {
        let about_provider = DefaultAboutProvider::new();
        println!("{}", about_provider.get_about_info());
        return Ok(());
    }

    if options.version {
        let about_provider = DefaultAboutProvider::new();
        let info = about_provider.get_about_info();
        println!("{} v{}", info.name, info.version);
        return Ok(());
    }

    let config_path = options
        .config_path
        .unwrap_or_else(|| std::path::PathBuf::from("gic.toml"));

    // 2. Load Configuration
    let config = ConfigLoader::load_from_file(&config_path)?;

    // 3. Initialize Structured Logging
    let _ = init_logging(&config.logging);
    info!(app_name = %config.app_name, "Starting GIC Terminal Engine");

    // 4. Initialize Terminal Engine (Full screen, alternate screen, mouse enabled)
    let mut engine_state = EngineState::new();
    let mut terminal_engine = TerminalEngine::new(engine_state.mouse_enabled)?;
    let (w, h) = terminal_engine.size().unwrap_or((80, 24));
    engine_state.metrics.update_dimensions(w, h);

    let event_stream = EventStream::new(&config.ui);
    let mut render_engine = RenderEngine::new(&config.ui);

    info!("Terminal Engine initialized. Entering main event & render loop.");

    // 5. Main Terminal Event & FPS-Independent Render Loop
    let shutdown_reason = loop {
        // Poll Events
        let event = event_stream.next_event()?;

        match event {
            InputEvent::Key(key) => {
                // Exit Shortcuts: 'q' or 'Ctrl+C'
                if key.code == KeyCode::Char('q')
                    || (key.modifiers.control && key.code == KeyCode::Char('c'))
                {
                    break ShutdownReason::UserRequested;
                }

                // Dynamic Mouse Capture Toggle: 'm'
                if key.code == KeyCode::Char('m') {
                    let mouse_on = engine_state.toggle_mouse();
                    if let Err(e) = terminal_engine.set_mouse_capture(mouse_on) {
                        engine_state.set_status(format!("Mouse Toggle Error: {}", e));
                    } else {
                        engine_state.set_status(format!(
                            "Mouse capture {}",
                            if mouse_on { "enabled" } else { "disabled" }
                        ));
                    }
                } else {
                    engine_state.set_status(format!("Key Pressed: {:?}", key.code));
                }
            }
            InputEvent::Mouse(mouse) => {
                let action_str = match mouse.action {
                    MouseAction::Press(btn) => format!("Click({:?})", btn),
                    MouseAction::Release(btn) => format!("Release({:?})", btn),
                    MouseAction::Drag(btn) => format!("Drag({:?})", btn),
                    MouseAction::Moved => "Moved".to_string(),
                    MouseAction::ScrollUp => "ScrollUp".to_string(),
                    MouseAction::ScrollDown => "ScrollDown".to_string(),
                };
                engine_state.set_status(format!(
                    "Mouse Event: {} at ({}, {})",
                    action_str, mouse.column, mouse.row
                ));
            }
            InputEvent::Resize { width, height } => {
                engine_state.metrics.update_dimensions(width, height);
                engine_state.set_status(format!("Window Resized to {}x{}", width, height));
            }
            InputEvent::Tick => {
                engine_state.metrics.record_tick();
            }
            InputEvent::Paste(ref text) => {
                engine_state.set_status(format!("Pasted {} characters", text.len()));
            }
        }

        // Render Frame when Frame Budget permits (FPS-independent rendering)
        if render_engine.should_render() {
            terminal_engine.terminal_mut().draw(|frame| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(1),
                    ])
                    .split(frame.size());

                // Header Component
                let header = Paragraph::new("GIC – General Infrastructure Console")
                    .style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Milestone 2: Terminal Engine "),
                    );
                frame.render_widget(header, chunks[0]);

                // Body Canvas Component
                let body_text = format!(
                    "Terminal Engine Active\n\n\
                     • Full Screen & Alternate Screen Buffer: Active\n\
                     • FPS Target: {} FPS | Frame Count: {}\n\
                     • Tick Rate Target: {} ms | Tick Count: {}\n\
                     • Mouse Support: {}\n\n\
                     Controls:\n\
                     - Press 'm' to toggle Mouse Capture\n\
                     - Press 'q' or 'Ctrl+C' to Gracefully Exit\n\
                     - Click or Scroll anywhere to test Mouse Events",
                    config.ui.frame_rate_fps,
                    engine_state.metrics.frame_count,
                    config.ui.tick_rate_ms,
                    engine_state.metrics.tick_count,
                    if engine_state.mouse_enabled {
                        "ENABLED"
                    } else {
                        "DISABLED"
                    }
                );

                let body = Paragraph::new(body_text)
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Engine Console "),
                    );
                frame.render_widget(body, chunks[1]);

                // Modular Status Bar Component
                let status_bar = StatusBar::new(&engine_state);
                frame.render_widget(status_bar, chunks[2]);
            })?;

            render_engine.record_render(&mut engine_state.metrics);
        }
    };

    // 6. Graceful Exit (TerminalEngine drop restores raw mode and alternate screen)
    info!(reason = %shutdown_reason, "Exiting Terminal Engine cleanly");
    println!("GIC Engine shut down cleanly: {}", shutdown_reason);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_cli_options_integration() {
        let opts = CliOptions::parse_from(vec!["gic", "--config", "test.toml"]);
        assert_eq!(
            opts.config_path,
            Some(std::path::PathBuf::from("test.toml"))
        );
    }
}
