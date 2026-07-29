use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

use gic_core::{
    CursorPosition, Document, EditorMode, EditorState, InputEvent, KeyCode, ShutdownReason,
    TextBuffer, UIConfig,
};
use gic_tui::{
    event_stream::EventStream,
    render_engine::RenderEngine,
    renderer::{
        pipeline::RenderPipeline, render_state::RenderState as TuiRenderState, themes::builtin,
        viewport::Viewport,
    },
    terminal::TerminalEngine,
};

use gic_core::language_engine::{EngineDiagnostic, LanguageEngineRegistry};

#[derive(Debug, Default)]
struct AnimatedCursor {
    pub current_x: f64,
    pub current_y: f64,
    pub target_x: f64,
    pub target_y: f64,
    pub blink_phase: f64,
}

impl AnimatedCursor {
    pub fn update(&mut self, dt: f64) -> bool {
        let mut moving = false;

        // Spring-like ease-out
        let dx = self.target_x - self.current_x;
        let dy = self.target_y - self.current_y;

        if dx.abs() > 0.05 {
            self.current_x += dx * 20.0 * dt;
            moving = true;
        } else {
            self.current_x = self.target_x;
        }

        if dy.abs() > 0.05 {
            self.current_y += dy * 20.0 * dt;
            moving = true;
        } else {
            self.current_y = self.target_y;
        }

        // Blink phase 0.0 -> 1.0 -> 0.0
        self.blink_phase = (self.blink_phase + dt * 2.0) % 2.0;

        // Always return true because we want to animate blink even if not moving
        true
    }
}

pub struct EditorApp {
    state: EditorState,
    ui_config: UIConfig,
    language_registry: LanguageEngineRegistry,
    cached_diagnostics: Vec<EngineDiagnostic>,
    cached_completions: Vec<gic_core::language_engine::Completion>,
    cached_ghost_text: Option<String>,
    cached_hover: Option<gic_core::language_engine::HoverInfo>,
    animated_cursor: AnimatedCursor,
    last_tick: std::time::Instant,
}

impl EditorApp {
    /// Constructs a new `EditorApp`, loading the file if specified.
    pub fn new(file_path: Option<PathBuf>, ui_config: UIConfig, debug_mode: bool) -> Self {
        let mut document = Document::new_empty();
        let buffer = if let Some(ref path) = file_path {
            let path_str = path.to_string_lossy().to_string();
            document.set_path(&path_str);
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    document.mark_saved();
                    TextBuffer::from_str(&content)
                }
                Err(e) => {
                    info!("Could not open file {:?}: {}", path, e);
                    TextBuffer::new()
                }
            }
        } else {
            TextBuffer::new()
        };

        let project_root = file_path
            .as_ref()
            .and_then(|p| gic_core::workspace::detect_project_root(p));
        let mut workspace = gic_core::workspace::WorkspaceState::new(project_root);
        let buf_id = workspace.add_buffer(document, buffer);
        workspace.open_pane(buf_id);
        let mut state = EditorState::new(workspace, debug_mode);

        let display_name = state
            .document()
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[Untitled]")
            .to_string();

        let status = if state.document().path.is_some() {
            format!("Opened file: {}", display_name)
        } else {
            "New File — GIC Infrastructure Editor".to_string()
        };
        state.set_mode(EditorMode::Normal, &status);

        let mut app = Self {
            state,
            ui_config,
            language_registry: LanguageEngineRegistry::new(),
            cached_diagnostics: Vec::new(),
            cached_completions: Vec::new(),
            cached_ghost_text: None,
            cached_hover: None,
            animated_cursor: AnimatedCursor::default(),
            last_tick: std::time::Instant::now(),
        };
        app.update_diagnostics();
        app.update_completions();
        app.update_hover();
        app
    }

    fn update_diagnostics(&mut self) {
        let file_name = self
            .state
            .document()
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let file_ext = self
            .state
            .document()
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        let text = self.state.buffer().text();
        let engine = self
            .language_registry
            .resolve_with_content(&file_name, &file_ext, &text);
        self.cached_diagnostics = engine.diagnostics(&text);

        // Run spell checker
        let spell_diags =
            gic_core::language_engine::spell_checker::check_spelling(&text, engine.id());
        self.cached_diagnostics.extend(spell_diags);

        // Contextual Suggestions (Non-blocking)
        if engine.id() == "kubernetes" && text.contains("kind: Deployment") {
            if !text.contains("kind: Service") {
                self.cached_diagnostics
                    .push(gic_core::language_engine::EngineDiagnostic {
                        row: 0,
                        col: 0,
                        length: 0,
                        severity: gic_core::language_engine::EngineSeverity::Hint,
                        message:
                            "Missing: Service, Ingress. [Press Alt+G to Generate Missing Resources]"
                                .to_string(),
                        code: Some("GIC-SUGGEST".to_string()),
                        source: "gic-assistant".to_string(),
                        quick_fixes: vec![],
                    });
            }
        } else if engine.id() == "docker" {
            // Very naive check for docker-compose.yml in same dir
            let has_compose = if let Some(path) = &self.state.document().path {
                let p = std::path::Path::new(path);
                if let Some(parent) = p.parent() {
                    parent.join("docker-compose.yml").exists()
                        || parent.join("compose.yml").exists()
                } else {
                    false
                }
            } else {
                false
            };
            if !has_compose {
                self.cached_diagnostics.push(gic_core::language_engine::EngineDiagnostic {
                    row: 0,
                    col: 0,
                    length: 0,
                    severity: gic_core::language_engine::EngineSeverity::Hint,
                    message: "Related files not found: docker-compose.yml, .dockerignore. [Press Alt+G to Generate]".to_string(),
                    code: Some("GIC-SUGGEST".to_string()),
                    source: "gic-assistant".to_string(),
                    quick_fixes: vec![],
                });
            }
        }

        // Sort combined diagnostics
        self.cached_diagnostics
            .sort_by(|a, b| a.row.cmp(&b.row).then(a.col.cmp(&b.col)));
    }

    fn update_completions(&mut self) {
        if self.state.mode != EditorMode::Insert {
            self.cached_completions.clear();
            return;
        }
        let file_name = self
            .state
            .document()
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let file_ext = self
            .state
            .document()
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        let text = self.state.buffer().text();
        let engine = self
            .language_registry
            .resolve_with_content(&file_name, &file_ext, &text);
        let cursor = self.state.cursor();
        self.cached_completions = engine.completions(&text, cursor.row, cursor.col);

        self.cached_ghost_text = None;
        if let Some(line) = self.state.buffer().lines().get(cursor.row) {
            let line_up_to_cursor = &line[..cursor.col];
            // The word the cursor is currently touching (or empty string if on a space/symbol)
            let current_word = line_up_to_cursor
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .last()
                .unwrap_or("");

            for comp in &self.cached_completions {
                // Ignore empty completions
                if comp.insert_text.is_empty() {
                    continue;
                }

                let insert_trimmed = comp.insert_text.trim_start();

                // If the user is typing the beginning of this completion, or hasn't started the word yet
                if insert_trimmed.starts_with(current_word) {
                    // Only show ghost text if there's actually something to complete
                    let ghost = &insert_trimmed[current_word.len()..];
                    if !ghost.is_empty() {
                        self.cached_ghost_text = Some(ghost.to_string());
                        break;
                    }
                }
            }
        }
    }

    fn jump_to_next_placeholder(&mut self) -> bool {
        let cursor = self.state.cursor();
        let start_row = cursor.row;
        let start_col = cursor.col;

        let mut found = None;

        let lines = self.state.buffer().lines().to_vec();

        for (i, line) in lines.iter().enumerate().skip(start_row) {
            let byte_offset = if i == start_row {
                line.chars().take(start_col).map(|c| c.len_utf8()).sum()
            } else {
                0
            };

            if byte_offset <= line.len() {
                if let Some(pos) = line[byte_offset..].find('█') {
                    let actual_byte_pos = byte_offset + pos;
                    let char_col = line[..actual_byte_pos].chars().count();
                    found = Some((i, char_col));
                    break;
                }
            }
        }

        if found.is_none() {
            for (i, line) in lines.iter().enumerate().take(start_row + 1) {
                let max_byte = if i == start_row {
                    line.chars().take(start_col).map(|c| c.len_utf8()).sum()
                } else {
                    line.len()
                };

                if max_byte <= line.len() {
                    if let Some(pos) = line[..max_byte].find('█') {
                        let char_col = line[..pos].chars().count();
                        found = Some((i, char_col));
                        break;
                    }
                }
            }
        }

        if let Some((r, c)) = found {
            let buffer = self.state.buffer_mut();
            buffer.cursor_mut().position = gic_core::CursorPosition::new(r, c);

            buffer.selection_mut().start(
                gic_core::CursorPosition::new(r, c),
                gic_core::buffer::selection::SelectionMode::Character,
            );
            buffer
                .selection_mut()
                .update(gic_core::CursorPosition::new(r, c + 1));

            self.sync_cursor_from_buffer();
            self.state.dirty.mark_full();
            self.state.force_autocomplete = true;
            return true;
        }

        false
    }

    fn apply_quick_fix(&mut self) -> bool {
        let cursor = self.state.cursor();
        if !self.state.quick_fix_menu_open {
            // Find diagnostic on current row with a quick fix
            if let Some(diag) = self
                .cached_diagnostics
                .iter()
                .find(|d| d.row == cursor.row && !d.quick_fixes.is_empty())
            {
                self.state.quick_fix_menu_open = true;
                self.state.quick_fix_selected_index = 0;
                self.state.dirty.mark_full(); // Force redraw for popup
                return true;
            } else {
                self.state
                    .engine
                    .set_status("No Quick Fix available for this line".to_string());
                return false;
            }
        }

        // Apply the chosen fix
        let fix = if let Some(diag) = self
            .cached_diagnostics
            .iter()
            .find(|d| d.row == cursor.row && !d.quick_fixes.is_empty())
        {
            if self.state.quick_fix_selected_index < diag.quick_fixes.len() {
                diag.quick_fixes[self.state.quick_fix_selected_index].clone()
            } else {
                return false;
            }
        } else {
            return false;
        };

        let buffer = self.state.buffer_mut();
        let start = gic_core::CursorPosition::new(fix.row, fix.col);
        let end = gic_core::CursorPosition::new(fix.row, fix.col + fix.replace_length);

        let _ = buffer.replace_range(start, end, &fix.new_text);

        self.state.quick_fix_menu_open = false;

        self.sync_cursor_from_buffer();
        self.state.document_mut().mark_modified();
        self.state.dirty.mark_full(); // Full redraw because lines might have shifted

        if fix.new_text.contains('█') {
            self.jump_to_next_placeholder();
        }

        self.update_diagnostics();
        self.update_completions();
        self.update_hover();

        self.state
            .engine
            .set_status(format!("✓ Applied Quick Fix: {}", fix.title));
        return true;
    }

    fn update_hover(&mut self) {
        let file_name = self
            .state
            .document()
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let file_ext = self
            .state
            .document()
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        let text = self.state.buffer().text();
        let engine = self
            .language_registry
            .resolve_with_content(&file_name, &file_ext, &text);
        let cursor = self.state.cursor();
        self.cached_hover = engine.hover(&text, cursor.row, cursor.col);
    }

    pub fn set_file_explorer_open(&mut self, open: bool) {
        self.state.file_explorer_open = open;
    }

    /// Runs the interactive editor event and render loop.
    pub fn run(mut self) -> Result<()> {
        let mut terminal_engine = TerminalEngine::new(self.state.engine.mouse_enabled)?;
        let (w, h) = terminal_engine.size().unwrap_or((80, 24));
        self.state.engine.metrics.update_dimensions(w, h);

        let event_stream = EventStream::new(&self.ui_config);
        let mut render_engine = RenderEngine::new(&self.ui_config);
        let mut pipeline = RenderPipeline::new();
        let theme = builtin::gic_dark();

        info!("Entering GIC Editor interactive loop");

        // Initialize viewport
        let initial_text_height = h.saturating_sub(1) as usize;
        let mut viewport = Viewport::new(
            initial_text_height,
            w as usize,
            self.state.buffer_mut().line_count(),
        );

        // Initial full redraw
        self.state.dirty.mark_full();

        let shutdown_reason = loop {
            if render_engine.should_render(&self.state.dirty) {
                let (width, height) = terminal_engine.size().unwrap_or((80, 24));
                let total_lines = self.state.buffer_mut().line_count();

                let text_height = height.saturating_sub(1) as usize;

                // Update persistent viewport
                viewport.resize(text_height, width as usize);
                viewport.set_total_lines(total_lines);
                viewport.ensure_cursor_visible(self.state.cursor().row, self.state.cursor().col, 2);

                if let Some(pane) = self.state.workspace.active_pane_mut() {
                    pane.scroll_row = viewport.scroll_row();
                    pane.scroll_col = viewport.scroll_col();
                }

                let language = self
                    .state
                    .document()
                    .path
                    .as_ref()
                    .map(|p| detect_language(p))
                    .unwrap_or("Plain Text");

                let render_state = TuiRenderState::new(&self.state, &theme)
                    .with_diagnostics(self.cached_diagnostics.clone())
                    .with_completions(self.cached_completions.clone())
                    .with_ghost_text(self.cached_ghost_text.clone());

                let render_state = if let Some(hover) = &self.cached_hover {
                    render_state.with_hover(hover.clone())
                } else {
                    render_state
                };

                if !self.state.search_matches.is_empty() {
                    // search results handled in pipeline
                }

                let mut cursor_render_info = None;

                terminal_engine.terminal_mut().draw(|frame| {
                    let area = frame.size();
                    if let Ok(info) = pipeline.render(frame.buffer_mut(), area, &render_state) {
                        cursor_render_info = Some(info.clone());
                        if info.visible {
                            frame.set_cursor(
                                info.screen_position.col as u16,
                                info.screen_position.row as u16,
                            );
                        }
                    }
                })?;

                // We don't use hardware cursor anymore because we rendered our own animated one!
                // if let Some(info) = cursor_render_info { ... }

                render_engine.record_render(&mut self.state.engine.metrics);
                self.state.dirty.clear();
            }

            let event = event_stream.next_event()?;

            match event {
                InputEvent::Key(key) => {
                    self.animated_cursor.blink_phase = 0.5; // Reset blink on keypress
                    if let Some(reason) = self.handle_key_input(key.code, key.modifiers) {
                        break reason;
                    }
                }
                InputEvent::Resize { width, height } => {
                    self.state.engine.metrics.update_dimensions(width, height);
                    self.state.dirty.mark_full();
                }
                InputEvent::Tick => {
                    let now = std::time::Instant::now();
                    let dt = now.duration_since(self.last_tick).as_secs_f64();
                    self.last_tick = now;

                    if self.animated_cursor.update(dt) {
                        self.state.dirty.mark_full(); // Force redraw for animation
                    }
                }
                InputEvent::Mouse(_) | InputEvent::Paste(_) => {}
            }
        };

        info!(reason = %shutdown_reason, "Exiting GIC Editor");
        println!("GIC Editor shut down: {}", shutdown_reason);
        Ok(())
    }

    /// Handles keyboard input based on current editor mode.
    fn handle_key_input(
        &mut self,
        code: KeyCode,
        modifiers: gic_core::KeyModifiers,
    ) -> Option<ShutdownReason> {
        let control = modifiers.control;
        let alt = modifiers.alt;

        if control && code == KeyCode::Char('q') {
            let is_modified =
                self.state.document_mut().is_modified || self.state.buffer_mut().is_modified();
            let warn_msg = "Warning: Unsaved changes. Press Ctrl+Q again to force quit.";
            if is_modified && self.state.engine.status_message != warn_msg {
                self.state.engine.set_status(warn_msg.to_string());
                self.state.dirty.mark_status();
                return None;
            }
            return Some(ShutdownReason::UserRequested);
        }
        if control && code == KeyCode::Char('s') {
            self.execute_command("w");
            return None;
        }
        if control && code == KeyCode::Char('z') {
            let _ = self.state.buffer_mut().undo();
            self.sync_cursor_from_buffer();
            self.state.dirty.mark_full();
            return None;
        }
        if control && code == KeyCode::Char('y') {
            let _ = self.state.buffer_mut().redo();
            self.sync_cursor_from_buffer();
            self.state.dirty.mark_full();
            return None;
        }
        if control && code == KeyCode::Char('f') {
            self.state.search_query.clear();
            self.state.search_matches.clear();
            self.state.set_mode(EditorMode::Search, "Search: ");
            return None;
        }
        if control && code == KeyCode::Char('r') {
            self.state.replace_query.clear();
            self.state.set_mode(EditorMode::Replace, "Replace with: ");
            return None;
        }
        if alt && code == KeyCode::Enter || (control && code == KeyCode::Char('.')) {
            self.apply_quick_fix();
            return None;
        }

        if alt && code == KeyCode::Char('g') {
            // Apply contextual suggestion
            let mut suggestion_handled = false;
            for diag in &self.cached_diagnostics {
                if let Some(code) = &diag.code {
                    if code == "GIC-SUGGEST" {
                        if diag.message.contains("Service, Ingress") {
                            // Append Service and Ingress boilerplate
                            for _ in 0..10000 {
                                self.state.buffer_mut().move_down();
                            }
                            self.state.buffer_mut().move_to_line_end();
                            let boilerplate = "\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: generated-service\nspec:\n  ports:\n    - port: 80\n  selector:\n    app: generated\n---\napiVersion: networking.k8s.io/v1\nkind: Ingress\nmetadata:\n  name: generated-ingress\nspec:\n  rules:\n    - host: generated.example.com\n      http:\n        paths:\n          - path: /\n            pathType: Prefix\n            backend:\n              service:\n                name: generated-service\n                port:\n                  number: 80\n";
                            let _ = self.state.buffer_mut().insert_str(boilerplate);
                            self.state.document_mut().mark_modified();
                            self.state.dirty.mark_full();
                            self.sync_cursor_from_buffer();
                            self.state.engine.set_status(
                                "Generated Missing Resources: Service, Ingress".to_string(),
                            );
                            suggestion_handled = true;
                            break;
                        } else if diag.message.contains("docker-compose.yml") {
                            if let Some(path) = &self.state.document().path {
                                let p = std::path::Path::new(path);
                                if let Some(parent) = p.parent() {
                                    let _ = std::fs::write(parent.join("docker-compose.yml"), "services:\n  app:\n    build: .\n    ports:\n      - \"8080:8080\"\n");
                                    let _ = std::fs::write(
                                        parent.join(".dockerignore"),
                                        "node_modules\ntarget\n.git\n",
                                    );
                                    self.state.engine.set_status("Generated related files: docker-compose.yml, .dockerignore".to_string());
                                    suggestion_handled = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if suggestion_handled {
                self.update_diagnostics();
            }
            return None;
        }

        match self.state.mode {
            EditorMode::Search => match code {
                KeyCode::Esc => {
                    self.state.search_query.clear();
                    self.state.search_matches.clear();
                    self.state.set_mode(EditorMode::Normal, "-- NORMAL --");
                }
                KeyCode::Char(ch) => {
                    self.state.search_query.push(ch);
                    self.state
                        .engine
                        .set_status(format!("Search: {}", self.state.search_query));
                    self.perform_search();
                }
                KeyCode::Backspace => {
                    self.state.search_query.pop();
                    self.state
                        .engine
                        .set_status(format!("Search: {}", self.state.search_query));
                    self.perform_search();
                }
                KeyCode::Enter => {
                    // Move to next match
                    self.next_search_match();
                }
                _ => {}
            },

            EditorMode::Replace => match code {
                KeyCode::Esc => {
                    self.state.replace_query.clear();
                    self.state.set_mode(EditorMode::Normal, "-- NORMAL --");
                }
                KeyCode::Char(ch) => {
                    self.state.replace_query.push(ch);
                    self.state
                        .engine
                        .set_status(format!("Replace with: {}", self.state.replace_query));
                }
                KeyCode::Backspace => {
                    self.state.replace_query.pop();
                    self.state
                        .engine
                        .set_status(format!("Replace with: {}", self.state.replace_query));
                }
                KeyCode::Enter => {
                    let replace_text = self.state.replace_query.clone();
                    let query_len = self.state.search_query.len();

                    if !self.state.search_matches.is_empty() && query_len > 0 {
                        let cursor = self.state.cursor();
                        let start = cursor;
                        let end = gic_core::CursorPosition::new(cursor.row, cursor.col + query_len);

                        let _ = self
                            .state
                            .buffer_mut()
                            .replace_range(start, end, &replace_text);
                        self.state.dirty.mark_full();

                        self.perform_search();
                        if !self.state.search_matches.is_empty() {
                            self.next_search_match();
                            self.state.set_mode(
                                EditorMode::Search,
                                &format!("Search: {}", self.state.search_query),
                            );
                        } else {
                            self.state.set_mode(EditorMode::Normal, "-- NORMAL --");
                        }
                    } else {
                        self.state.set_mode(EditorMode::Normal, "-- NORMAL --");
                    }
                }
                _ => {}
            },

            EditorMode::Visual => {} // TODO

            EditorMode::Normal => match code {
                KeyCode::Char('i') => {
                    self.state.set_mode(EditorMode::Insert, "-- INSERT --");
                }
                KeyCode::Char('I') => {
                    self.state.buffer_mut().move_to_line_start();
                    self.sync_cursor_from_buffer();
                    self.state.set_mode(EditorMode::Insert, "-- INSERT --");
                }
                KeyCode::Char('a') => {
                    let line_len = self.line_length(self.state.cursor().row);
                    if self.state.cursor().col < line_len {
                        self.state.cursor().col += 1;
                    }
                    self.state.set_mode(EditorMode::Insert, "-- INSERT --");
                }
                KeyCode::Char('A') => {
                    self.state.cursor().col = self.line_length(self.state.cursor().row);
                    self.state.set_mode(EditorMode::Insert, "-- INSERT --");
                }
                KeyCode::Char('o') => {
                    self.state.cursor().row += 1;
                    self.state.cursor().col = 0;
                    self.sync_buffer_cursor();
                    let _ = self.state.buffer_mut().insert_newline();
                    let pos = self.state.buffer_mut().cursor_position();
                    self.state.set_cursor(pos);
                    self.state.document_mut().mark_modified();
                    self.state.set_mode(EditorMode::Insert, "-- INSERT --");
                    self.state.dirty.mark_full(); // full redraw because lines shifted
                }
                KeyCode::Char('O') => {
                    self.state.cursor().col = 0;
                    self.sync_buffer_cursor();
                    let _ = self.state.buffer_mut().insert_newline();
                    let pos = self.state.buffer_mut().cursor_position();
                    self.state.set_cursor(pos);
                    self.state.document_mut().mark_modified();
                    self.state.set_mode(EditorMode::Insert, "-- INSERT --");
                    self.state.dirty.mark_full(); // full redraw because lines shifted
                }
                KeyCode::Char(':') => {
                    self.state.command_input.clear();
                    self.state.set_mode(EditorMode::Command, ":");
                }
                KeyCode::F(8) => {
                    // Jump to next diagnostic
                    let curr_row = self.state.cursor().row;
                    let curr_col = self.state.cursor().col;
                    let mut next_diag = None;
                    for diag in &self.cached_diagnostics {
                        if diag.row > curr_row || (diag.row == curr_row && diag.col > curr_col) {
                            next_diag = Some((diag.row, diag.col, diag.severity.clone()));
                            break;
                        }
                    }
                    if next_diag.is_none() {
                        if let Some(diag) = self.cached_diagnostics.first() {
                            next_diag = Some((diag.row, diag.col, diag.severity.clone()));
                        }
                    }

                    if let Some((row, col, severity)) = next_diag {
                        self.state.cursor().row = row;
                        self.state.cursor().col = col;
                        self.sync_buffer_cursor();
                        self.state.dirty.mark_full();
                        self.update_hover();
                        self.state
                            .engine
                            .set_status(format!("Jumped to {:?} (F8)", severity));
                    } else {
                        self.state
                            .engine
                            .set_status("No diagnostics in file.".to_string());
                    }
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    self.state.buffer_mut().move_left();
                    self.sync_cursor_from_buffer();
                    self.state.dirty.mark_status();
                    self.update_hover();
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    self.state.buffer_mut().move_right();
                    self.sync_cursor_from_buffer();
                    self.state.dirty.mark_status();
                    self.update_hover();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if self.state.quick_fix_menu_open {
                        if let Some(diag) = self
                            .cached_diagnostics
                            .iter()
                            .find(|d| d.row == self.state.cursor().row && !d.quick_fixes.is_empty())
                        {
                            if self.state.quick_fix_selected_index + 1 < diag.quick_fixes.len() {
                                self.state.quick_fix_selected_index += 1;
                                self.state.dirty.mark_full();
                            }
                        }
                    } else {
                        self.state.buffer_mut().move_down();
                        self.sync_cursor_from_buffer();
                        self.state.dirty.mark_full(); // lazy: full redraw on v-scroll
                        self.update_hover();
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if self.state.quick_fix_menu_open {
                        if self.state.quick_fix_selected_index > 0 {
                            self.state.quick_fix_selected_index -= 1;
                            self.state.dirty.mark_full();
                        }
                    } else {
                        self.state.buffer_mut().move_up();
                        self.sync_cursor_from_buffer();
                        self.state.dirty.mark_full(); // lazy: full redraw on v-scroll
                        self.update_hover();
                    }
                }
                KeyCode::Esc => {
                    if self.state.quick_fix_menu_open {
                        self.state.quick_fix_menu_open = false;
                        self.state.dirty.mark_full();
                    }
                }
                KeyCode::Enter => {
                    if self.state.quick_fix_menu_open {
                        self.apply_quick_fix();
                    }
                }
                KeyCode::PageDown => {
                    // Approximate PageDown by jumping 10 lines
                    for _ in 0..10 {
                        self.state.buffer_mut().move_down();
                    }
                    self.sync_cursor_from_buffer();
                    self.state.dirty.mark_full();
                }
                KeyCode::PageUp => {
                    // Approximate PageUp by jumping 10 lines
                    for _ in 0..10 {
                        self.state.buffer_mut().move_up();
                    }
                    self.sync_cursor_from_buffer();
                    self.state.dirty.mark_full();
                }
                KeyCode::Home | KeyCode::Char('0') => {
                    self.state.buffer_mut().move_to_line_start();
                    self.sync_cursor_from_buffer();
                    self.state.dirty.mark_status();
                }
                KeyCode::End | KeyCode::Char('$') => {
                    self.state.buffer_mut().move_to_line_end();
                    self.sync_cursor_from_buffer();
                    self.state.dirty.mark_status();
                }
                KeyCode::Char('x') => {
                    self.sync_buffer_cursor();
                    if self.state.buffer_mut().delete_char().is_ok() {
                        let pos = self.state.buffer_mut().cursor_position();
                        self.state.set_cursor(pos);
                        self.state.document_mut().mark_modified();
                        self.state
                            .engine
                            .set_status("Character deleted".to_string());
                        self.state.dirty.mark_line(self.state.cursor().row);
                    }
                }
                _ => {}
            },

            EditorMode::Insert => match code {
                KeyCode::Esc => {
                    if !self.cached_completions.is_empty() {
                        self.cached_completions.clear();
                        self.cached_ghost_text = None;
                        self.state.dirty.mark_full();
                    } else {
                        self.state.set_mode(EditorMode::Normal, "-- NORMAL --");
                    }
                }
                KeyCode::Char(ch) => {
                    if !self.cached_completions.is_empty() {
                        self.state.autocomplete_selected_index = 0;
                        self.state.autocomplete_scroll_offset = 0;
                    }
                    self.sync_buffer_cursor();

                    // Auto-closing brackets and quotes
                    let mut skip_insert = false;
                    let mut insert_closing = None;

                    let cursor = self.state.cursor();
                    let next_char = self
                        .state
                        .buffer_mut()
                        .lines()
                        .get(cursor.row)
                        .and_then(|line| line.chars().nth(cursor.col));

                    match ch {
                        '{' => insert_closing = Some('}'),
                        '[' => insert_closing = Some(']'),
                        '(' => insert_closing = Some(')'),
                        '"' => {
                            if next_char == Some('"') {
                                skip_insert = true;
                            } else {
                                insert_closing = Some('"');
                            }
                        }
                        '\'' => {
                            if next_char == Some('\'') {
                                skip_insert = true;
                            } else {
                                insert_closing = Some('\'');
                            }
                        }
                        '}' | ']' | ')' => {
                            if next_char == Some(ch) {
                                skip_insert = true;
                            }
                        }
                        _ => {}
                    }

                    if skip_insert {
                        let _ = self.state.buffer_mut().move_right();
                        let pos = self.state.buffer_mut().cursor_position();
                        self.state.set_cursor(pos);
                    } else if self.state.buffer_mut().insert_char(ch).is_ok() {
                        if let Some(closing) = insert_closing {
                            let _ = self.state.buffer_mut().insert_char(closing);
                            let _ = self.state.buffer_mut().move_left();
                        }
                        let pos = self.state.buffer_mut().cursor_position();
                        self.state.set_cursor(pos);
                        self.state.document_mut().mark_modified();
                        self.state.dirty.mark_line(self.state.cursor().row);
                    }
                    self.update_diagnostics();
                    self.update_completions();
                    self.update_hover();
                }
                KeyCode::Enter => {
                    if !self.cached_completions.is_empty() {
                        let comp =
                            self.cached_completions[self.state.autocomplete_selected_index].clone();
                        self.cached_completions.clear();
                        self.cached_ghost_text = None;

                        // We need to delete the typed prefix if any, but since the engine doesn't tell us the prefix length directly,
                        // and `insert_text` is just the suffix in some engines or the full text, wait, standard LSPs provide the full text
                        // and a text edit range. But our Completion struct only has `insert_text` and `label`.
                        // If it's a suffix, we just insert. Let's just insert it.
                        let _ = self.state.buffer_mut().insert_str(&comp.insert_text);
                        self.sync_cursor_from_buffer();
                        self.state.document_mut().mark_modified();
                        self.state.dirty.mark_full();

                        if comp.insert_text.contains('█') {
                            self.jump_to_next_placeholder();
                        }

                        self.update_diagnostics();
                        self.update_hover();
                        return None;
                    }

                    self.sync_buffer_cursor();

                    let cursor = self.state.cursor();
                    let path = self.state.document().path.clone();
                    let ext = path
                        .as_ref()
                        .and_then(|p| p.extension())
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_string();
                    let fname = path
                        .as_ref()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    let engine = self
                        .language_registry
                        .resolve_with_content(&fname, &ext, "");

                    let mut smart_enter_str = None;
                    let (mut leading_whitespace, extra_indent) =
                        if let Some(line) = self.state.buffer_mut().lines().get(cursor.row) {
                            let ws = line
                                .chars()
                                .take_while(|c| c.is_whitespace())
                                .collect::<String>();
                            let trimmed = line[..cursor.col].trim_end();

                            smart_enter_str = engine.smart_enter(trimmed);

                            let mut extra = 0;
                            if trimmed.ends_with('{')
                                || trimmed.ends_with('[')
                                || trimmed.ends_with('(')
                            {
                                extra = 4;
                            } else if (ext == "yaml" || ext == "yml") && trimmed.ends_with(':') {
                                extra = 2;
                            } else if ext == "sh" || ext == "bash" {
                                if trimmed.ends_with("then")
                                    || trimmed.ends_with("do")
                                    || trimmed.ends_with("else")
                                    || trimmed.ends_with('\\')
                                {
                                    extra = 4;
                                }
                            } else if fname == "Dockerfile" && trimmed.ends_with('\\') {
                                extra = 4;
                            }

                            (ws, extra)
                        } else {
                            (String::new(), 0)
                        };

                    if extra_indent > 0 {
                        leading_whitespace.push_str(&" ".repeat(extra_indent));
                    }

                    if self.state.buffer_mut().insert_newline().is_ok() {
                        if !leading_whitespace.is_empty() {
                            let _ = self.state.buffer_mut().insert_str(&leading_whitespace);
                        }
                        if let Some(smart_str) = smart_enter_str {
                            let _ = self.state.buffer_mut().insert_str(&smart_str);
                        }
                        self.sync_cursor_from_buffer();
                        self.state.document_mut().mark_modified();
                        self.state.dirty.mark_full(); // shifted lines
                        self.update_diagnostics();
                        self.update_completions();
                        self.update_hover();
                    }
                }
                KeyCode::Backspace => {
                    self.sync_buffer_cursor();
                    let old_row = self.state.cursor().row;
                    if self.state.buffer_mut().delete_backspace().is_ok() {
                        self.sync_cursor_from_buffer();
                        self.state.document_mut().mark_modified();
                        if self.state.cursor().row != old_row {
                            self.state.dirty.mark_full();
                        } else {
                            self.state.dirty.mark_line(self.state.cursor().row);
                        }
                        self.update_diagnostics();
                        self.update_completions();
                        self.update_hover();
                    }
                }
                KeyCode::Delete => {
                    self.sync_buffer_cursor();
                    if self.state.buffer_mut().delete_char().is_ok() {
                        self.sync_cursor_from_buffer();
                        self.state.document_mut().mark_modified();
                        self.state.dirty.mark_line(self.state.cursor().row);
                        self.update_diagnostics();
                    }
                }
                KeyCode::Tab => {
                    if !self.cached_completions.is_empty() {
                        let comp =
                            self.cached_completions[self.state.autocomplete_selected_index].clone();
                        self.cached_completions.clear();
                        self.cached_ghost_text = None;

                        let _ = self.state.buffer_mut().insert_str(&comp.insert_text);
                        self.sync_cursor_from_buffer();
                        self.state.document_mut().mark_modified();
                        self.state.dirty.mark_full();

                        if comp.insert_text.contains('█') {
                            self.jump_to_next_placeholder();
                        }

                        self.update_diagnostics();
                        self.update_hover();
                        return None;
                    }

                    self.sync_buffer_cursor();

                    let cursor = self.state.cursor();
                    let path = self.state.document().path.clone();
                    let ext = path
                        .as_ref()
                        .and_then(|p| p.extension())
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_string();
                    let fname = path
                        .as_ref()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    let engine = self
                        .language_registry
                        .resolve_with_content(&fname, &ext, "");

                    let mut template = None;
                    let mut word_start = 0;
                    if let Some(line) = self.state.buffer().lines().get(cursor.row) {
                        let text_before_cursor = &line[..cursor.col];
                        let last_word = text_before_cursor.split_whitespace().last().unwrap_or("");
                        if !last_word.is_empty() {
                            template = engine.template_expansion(last_word);
                            word_start = cursor.col - last_word.len();
                        }
                    }

                    if let Some(template_str) = template {
                        // Delete the keyword
                        for _ in 0..(cursor.col - word_start) {
                            let _ = self.state.buffer_mut().delete_backspace();
                        }
                        // Insert the template
                        let _ = self.state.buffer_mut().insert_str(&template_str);
                        self.sync_cursor_from_buffer();
                        self.state.document_mut().mark_modified();
                        self.state.dirty.mark_full(); // Full redraw in case of newlines

                        if template_str.contains('█') {
                            self.jump_to_next_placeholder();
                        }

                        self.update_diagnostics();
                        self.update_completions();
                        self.update_hover();
                    } else if let Some(ghost) = self.cached_ghost_text.clone() {
                        let _ = self.state.buffer_mut().insert_str(&ghost);
                        self.sync_cursor_from_buffer();
                        self.state.document_mut().mark_modified();
                        self.state.dirty.mark_full(); // Full redraw in case of newlines

                        if ghost.contains('█') {
                            self.jump_to_next_placeholder();
                        }

                        self.update_diagnostics();
                        self.update_completions();
                        self.update_hover();
                    } else if self.jump_to_next_placeholder() {
                        self.update_diagnostics();
                        self.update_completions();
                        self.update_hover();
                    } else if self.state.buffer_mut().insert_tab(4).is_ok() {
                        self.sync_cursor_from_buffer();
                        self.state.document_mut().mark_modified();
                        self.state.dirty.mark_full(); // Safer to full redraw
                        self.update_diagnostics();
                        self.update_completions();
                        self.update_hover();
                    }
                }
                KeyCode::Right => {
                    self.sync_buffer_cursor();
                    let line_len = self.line_length(self.state.cursor().row);
                    if self.state.cursor().col == line_len && self.cached_ghost_text.is_some() {
                        let ghost = self.cached_ghost_text.clone().unwrap();
                        let _ = self.state.buffer_mut().insert_str(&ghost);
                        self.sync_cursor_from_buffer();
                        self.state.document_mut().mark_modified();
                        self.state.dirty.mark_full();
                        self.update_diagnostics();
                        self.update_completions();
                        self.update_hover();
                    } else {
                        self.state.buffer_mut().move_right();
                        self.sync_cursor_from_buffer();
                        self.state.dirty.mark_status();
                        self.update_hover();
                    }
                }
                KeyCode::Left => {
                    self.state.buffer_mut().move_left();
                    self.sync_cursor_from_buffer();
                    self.state.dirty.mark_status();
                    self.update_hover();
                }
                KeyCode::Up => {
                    if !self.cached_completions.is_empty() {
                        if self.state.autocomplete_selected_index > 0 {
                            self.state.autocomplete_selected_index -= 1;
                            if self.state.autocomplete_selected_index
                                < self.state.autocomplete_scroll_offset
                            {
                                self.state.autocomplete_scroll_offset =
                                    self.state.autocomplete_selected_index;
                            }
                            self.state.dirty.mark_full();
                        }
                    } else {
                        self.state.buffer_mut().move_up();
                        self.sync_cursor_from_buffer();
                        self.state.dirty.mark_full(); // lazy update on vertical movement
                    }
                }
                KeyCode::Down => {
                    if !self.cached_completions.is_empty() {
                        if self.state.autocomplete_selected_index + 1
                            < self.cached_completions.len()
                        {
                            self.state.autocomplete_selected_index += 1;
                            let max_visible = 10;
                            if self.state.autocomplete_selected_index
                                >= self.state.autocomplete_scroll_offset + max_visible
                            {
                                self.state.autocomplete_scroll_offset =
                                    self.state.autocomplete_selected_index - max_visible + 1;
                            }
                            self.state.dirty.mark_full();
                        }
                    } else {
                        self.state.buffer_mut().move_down();
                        self.sync_cursor_from_buffer();
                        self.state.dirty.mark_full();
                    }
                }
                KeyCode::Home => {
                    if !self.cached_completions.is_empty() {
                        self.state.autocomplete_selected_index = 0;
                        self.state.autocomplete_scroll_offset = 0;
                        self.state.dirty.mark_full();
                    } else {
                        self.state.buffer_mut().move_to_line_start();
                        self.sync_cursor_from_buffer();
                        self.state.dirty.mark_status();
                    }
                }
                KeyCode::End => {
                    if !self.cached_completions.is_empty() {
                        self.state.autocomplete_selected_index =
                            self.cached_completions.len().saturating_sub(1);
                        let max_visible = 10;
                        if self.state.autocomplete_selected_index >= max_visible {
                            self.state.autocomplete_scroll_offset =
                                self.state.autocomplete_selected_index - max_visible + 1;
                        }
                        self.state.dirty.mark_full();
                    } else {
                        self.state.buffer_mut().move_to_line_end();
                        self.sync_cursor_from_buffer();
                        self.state.dirty.mark_status();
                    }
                }
                KeyCode::PageDown => {
                    if !self.cached_completions.is_empty() {
                        let max_visible = 10;
                        self.state.autocomplete_selected_index =
                            (self.state.autocomplete_selected_index + max_visible)
                                .min(self.cached_completions.len().saturating_sub(1));
                        if self.state.autocomplete_selected_index
                            >= self.state.autocomplete_scroll_offset + max_visible
                        {
                            self.state.autocomplete_scroll_offset = self
                                .state
                                .autocomplete_selected_index
                                .saturating_sub(max_visible - 1);
                        }
                        self.state.dirty.mark_full();
                    } else {
                        for _ in 0..10 {
                            self.state.buffer_mut().move_down();
                        }
                        self.sync_cursor_from_buffer();
                        self.state.dirty.mark_full();
                    }
                }
                KeyCode::PageUp => {
                    if !self.cached_completions.is_empty() {
                        let max_visible = 10;
                        self.state.autocomplete_selected_index = self
                            .state
                            .autocomplete_selected_index
                            .saturating_sub(max_visible);
                        if self.state.autocomplete_selected_index
                            < self.state.autocomplete_scroll_offset
                        {
                            self.state.autocomplete_scroll_offset =
                                self.state.autocomplete_selected_index;
                        }
                        self.state.dirty.mark_full();
                    } else {
                        for _ in 0..10 {
                            self.state.buffer_mut().move_up();
                        }
                        self.sync_cursor_from_buffer();
                        self.state.dirty.mark_full();
                    }
                }
                _ => {}
            },

            EditorMode::Command => match code {
                KeyCode::Esc => {
                    self.state.command_input.clear();
                    self.state.set_mode(EditorMode::Normal, "-- NORMAL --");
                }
                KeyCode::Char(ch) => {
                    self.state.command_input.push(ch);
                    self.state
                        .engine
                        .set_status(format!(":{}", self.state.command_input));
                    self.state.dirty.mark_status();
                }
                KeyCode::Backspace => {
                    self.state.command_input.pop();
                    self.state
                        .engine
                        .set_status(format!(":{}", self.state.command_input));
                    self.state.dirty.mark_status();
                }
                KeyCode::Enter => {
                    let cmd = self.state.command_input.trim().to_string();
                    self.state.command_input.clear();
                    self.state.set_mode(EditorMode::Normal, "-- NORMAL --");

                    return self.execute_command(&cmd);
                }
                _ => {}
            },
        }

        None
    }

    fn perform_search(&mut self) {
        self.state.search_matches.clear();
        if self.state.search_query.is_empty() {
            self.state.dirty.mark_full();
            return;
        }

        let query = self.state.search_query.clone();
        // Simple case-insensitive exact search
        let mut matches_to_add = Vec::new();
        for (row_idx, line) in self.state.buffer_mut().lines().iter().enumerate() {
            let lower_line = line.to_lowercase();
            let lower_query = query.to_lowercase();
            let mut start = 0;
            while let Some(idx) = lower_line[start..].find(&lower_query) {
                let actual_idx = start + idx;
                // Convert byte index to char index
                let char_idx = line[..actual_idx].chars().count();
                matches_to_add.push(gic_core::CursorPosition::new(row_idx, char_idx));
                start = actual_idx + lower_query.len();
            }
        }

        self.state.search_matches.extend(matches_to_add);
        if let Some(&first_match) = self.state.search_matches.first() {
            self.state.set_cursor(first_match);
            self.sync_buffer_cursor();
        }

        self.state.dirty.mark_full();
    }

    fn next_search_match(&mut self) {
        if self.state.search_matches.is_empty() {
            return;
        }

        let current_pos = self.state.cursor();
        // Find next match after current cursor
        let next = self
            .state
            .search_matches
            .iter()
            .find(|&&p| p > current_pos)
            .copied()
            .unwrap_or(self.state.search_matches[0]); // Wrap around

        self.state.set_cursor(next);
        self.sync_buffer_cursor();
        self.state.dirty.mark_full();
    }

    /// Helper to synchronize the TextBuffer's cursor back into the EditorState
    fn sync_cursor_from_buffer(&mut self) {
        let pos = self.state.buffer_mut().cursor_position();
        self.state.set_cursor(pos);
    }

    /// Synchronizes cursor position to `TextBuffer` internal cursor.
    fn sync_buffer_cursor(&mut self) {
        let cursor = self.state.cursor();
        self.state
            .buffer_mut()
            .set_cursor_position(cursor.row, cursor.col);
    }

    /// Helper to get line length safely.
    fn line_length(&mut self, row: usize) -> usize {
        self.state
            .buffer_mut()
            .line(row)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    /// Executes a `:` editor command.
    fn execute_command(&mut self, cmd: &str) -> Option<ShutdownReason> {
        match cmd {
            "w" => {
                let path_opt = self.state.document_mut().path.clone();
                if let Some(path) = path_opt {
                    let text = self.state.buffer_mut().text();
                    match std::fs::write(&path, text) {
                        Ok(_) => {
                            self.state.document_mut().mark_saved();
                            self.state
                                .engine
                                .set_status(format!("Written to {:?}", path));
                        }
                        Err(e) => {
                            self.state.engine.set_status(format!("Save error: {}", e));
                        }
                    }
                } else {
                    self.state
                        .engine
                        .set_status("Error: No file name (use :w <filename>)".to_string());
                }
            }
            "q" => {
                if self.state.document_mut().is_modified {
                    self.state.engine.set_status(
                        "Error: No write since last change (use :q! to override)".to_string(),
                    );
                } else {
                    return Some(ShutdownReason::UserRequested);
                }
            }
            "q!" => {
                return Some(ShutdownReason::UserRequested);
            }
            "wq" | "x" => {
                let path_opt = self.state.document_mut().path.clone();
                if let Some(path) = path_opt {
                    let text = self.state.buffer_mut().text();
                    let _ = std::fs::write(path, text);
                    self.state.document_mut().mark_saved();
                }
                return Some(ShutdownReason::UserRequested);
            }
            "fmt" => {
                let file_name = self
                    .state
                    .document()
                    .path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                let file_ext = self
                    .state
                    .document()
                    .path
                    .as_ref()
                    .and_then(|p| p.extension())
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_string();
                let text = self.state.buffer().text();
                let engine = self
                    .language_registry
                    .resolve_with_content(&file_name, &file_ext, &text);

                if let Some(formatted) = engine.format(&text) {
                    // Update buffer with formatted text
                    self.state.buffer_mut().set_text(&formatted);
                    self.state
                        .engine
                        .set_status("Formatted current file.".to_string());
                    self.state.dirty.mark_full();
                } else {
                    self.state.engine.set_status(
                        "No formatting changes needed or formatter not available.".to_string(),
                    );
                }
            }
            c if c.starts_with("w ") => {
                let target = c[2..].trim();
                if !target.is_empty() {
                    let path = PathBuf::from(target);
                    let text = self.state.buffer_mut().text();
                    match std::fs::write(&path, text) {
                        Ok(_) => {
                            self.state.document_mut().set_path(target);
                            self.state.document_mut().mark_saved();
                            self.state.engine.set_status(format!("Saved to {}", target));
                        }
                        Err(e) => {
                            self.state.engine.set_status(format!("Save error: {}", e));
                        }
                    }
                }
            }
            _ => {
                self.state
                    .engine
                    .set_status(format!("Unknown command: :{}", cmd));
            }
        }
        self.state.dirty.mark_status();
        None
    }
}

fn detect_language(path: &PathBuf) -> &'static str {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        match ext.to_lowercase().as_str() {
            "rs" => "Rust",
            "md" => "Markdown",
            "toml" => "TOML",
            "json" => "JSON",
            "yaml" | "yml" => "YAML",
            "tf" => "Terraform",
            "sh" | "bash" | "zsh" => "Shell",
            "c" | "h" => "C",
            "cpp" | "hpp" => "C++",
            "js" => "JavaScript",
            "ts" => "TypeScript",
            "py" => "Python",
            "go" => "Go",
            _ => "Plain Text",
        }
    } else if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
        if name.to_lowercase() == "dockerfile" {
            "Dockerfile"
        } else {
            "Plain Text"
        }
    } else {
        "Plain Text"
    }
}
