//! # Render State
use gic_core::EditorState;
use gic_core::language_engine::{EngineDiagnostic, HoverInfo};
use crate::renderer::file_explorer::FileTreeEntry;
use crate::renderer::themes::Theme;

pub struct RenderState<'a> {
    pub editor: &'a EditorState,
    pub theme: &'a Theme,
    pub file_explorer_open: bool,
    pub intelligence_panel_open: bool,
    pub bottom_panel_open: bool,
    pub command_palette_open: bool,
    pub diagnostics: Vec<EngineDiagnostic>,
    pub hover_info: Option<HoverInfo>,
    pub completions: Vec<gic_core::language_engine::Completion>,
    pub ghost_text: Option<String>,
    pub file_tree: Vec<FileTreeEntry>,
    pub explorer_selected: usize,
    pub explorer_scroll: usize,
}

impl<'a> RenderState<'a> {
    pub fn new(editor: &'a EditorState, theme: &'a Theme) -> Self {
        Self {
            editor,
            theme,
            file_explorer_open: editor.file_explorer_open,
            intelligence_panel_open: false,
            bottom_panel_open: false,
            command_palette_open: false,
            diagnostics: Vec::new(),
            hover_info: None,
            completions: Vec::new(),
            ghost_text: None,
            file_tree: Vec::new(),
            explorer_selected: 0,
            explorer_scroll: 0,
        }
    }

    pub fn with_diagnostics(mut self, diags: Vec<EngineDiagnostic>) -> Self {
        self.diagnostics = diags;
        self
    }

    pub fn with_hover(mut self, info: HoverInfo) -> Self {
        self.hover_info = Some(info);
        self
    }

    pub fn with_completions(mut self, comps: Vec<gic_core::language_engine::Completion>) -> Self {
        self.completions = comps;
        self
    }

    pub fn with_ghost_text(mut self, text: Option<String>) -> Self {
        self.ghost_text = text;
        self
    }

    pub fn with_file_tree(mut self, tree: Vec<FileTreeEntry>) -> Self {
        self.file_tree = tree;
        self
    }

    pub fn with_intelligence_panel(mut self, open: bool) -> Self {
        self.intelligence_panel_open = open;
        self
    }

    pub fn with_bottom_panel(mut self, open: bool) -> Self {
        self.bottom_panel_open = open;
        self
    }
}
