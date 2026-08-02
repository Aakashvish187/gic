use crate::{CursorPosition, Document, EngineState, TextBuffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Visual,
    Command,
    Search,
    Replace,
}

impl EditorMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Visual => "VISUAL",
            Self::Command => "COMMAND",
            Self::Search => "SEARCH",
            Self::Replace => "REPLACE",
        }
    }
}

impl Default for EditorMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// Tracking struct for rendering only what has changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyRegions {
    pub full_redraw: bool,
    pub status_bar: bool,
    pub lines: Vec<usize>,
}

impl Default for DirtyRegions {
    fn default() -> Self {
        Self {
            full_redraw: true,
            status_bar: true,
            lines: Vec::new(),
        }
    }
}

impl DirtyRegions {
    pub fn mark_full(&mut self) {
        self.full_redraw = true;
        self.status_bar = true;
        self.lines.clear();
    }

    pub fn mark_status(&mut self) {
        self.status_bar = true;
    }

    pub fn mark_line(&mut self, row: usize) {
        if !self.full_redraw && !self.lines.contains(&row) {
            self.lines.push(row);
        }
    }

    pub fn clear(&mut self) {
        self.full_redraw = false;
        self.status_bar = false;
        self.lines.clear();
    }
}

/// Central state object containing all editor domain state.
pub struct EditorState {
    pub mode: EditorMode,
    pub workspace: crate::workspace::WorkspaceState,
    pub engine: EngineState,
    pub dirty: DirtyRegions,
    pub debug_mode: bool,
    pub command_input: String,
    pub file_explorer_open: bool,
    pub intelligence_panel_open: bool,
    pub bottom_panel_open: bool,
    pub search_query: String,
    pub replace_query: String,
    pub search_matches: Vec<CursorPosition>,
    pub force_autocomplete: bool,
    pub autocomplete_selected_index: usize,
    pub autocomplete_scroll_offset: usize,
    pub quick_fix_menu_open: bool,
    pub quick_fix_selected_index: usize,
    pub validation_error_popup: Option<crate::language_engine::EngineDiagnostic>,
}

impl EditorState {
    pub fn new(mut workspace: crate::workspace::WorkspaceState, debug_mode: bool) -> Self {
        Self {
            mode: EditorMode::Normal,
            workspace,
            engine: EngineState::new(),
            dirty: DirtyRegions::default(),
            debug_mode,
            command_input: String::new(),
            file_explorer_open: false,
            intelligence_panel_open: true,
            bottom_panel_open: false,
            search_query: String::new(),
            replace_query: String::new(),
            search_matches: Vec::new(),
            force_autocomplete: false,
            autocomplete_selected_index: 0,
            autocomplete_scroll_offset: 0,
            quick_fix_menu_open: false,
            quick_fix_selected_index: 0,
            validation_error_popup: None,
        }
    }

    pub fn set_mode(&mut self, mode: EditorMode, status: &str) {
        self.mode = mode;
        self.engine.active_mode = mode.name().to_string();
        self.engine.set_status(status.to_string());
        self.dirty.mark_status();
    }

    pub fn buffer(&self) -> &TextBuffer {
        let buffer_id = self
            .workspace
            .active_pane_ref()
            .map(|p| p.buffer_id)
            .unwrap();
        self.workspace.buffers.get(&buffer_id).unwrap()
    }

    pub fn buffer_mut(&mut self) -> &mut TextBuffer {
        self.workspace.active_buffer_mut().unwrap()
    }

    pub fn document(&self) -> &Document {
        let buffer_id = self
            .workspace
            .active_pane_ref()
            .map(|p| p.buffer_id)
            .unwrap();
        self.workspace.documents.get(&buffer_id).unwrap()
    }

    pub fn document_mut(&mut self) -> &mut Document {
        self.workspace.active_document_mut().unwrap()
    }

    pub fn cursor(&self) -> CursorPosition {
        self.workspace
            .active_pane_ref()
            .map(|p| p.cursor)
            .unwrap_or_else(CursorPosition::zero)
    }

    pub fn set_cursor(&mut self, cursor: CursorPosition) {
        if let Some(pane) = self.workspace.active_pane_mut() {
            pane.cursor = cursor;
        }
    }
}
