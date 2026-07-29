use std::collections::HashMap;
use std::path::PathBuf;

use crate::{Document, TextBuffer};
use super::{BufferId, EditorPane};

/// Represents the global workspace state.
pub struct WorkspaceState {
    pub project_root: Option<PathBuf>,
    
    pub documents: HashMap<BufferId, Document>,
    pub buffers: HashMap<BufferId, TextBuffer>,
    
    pub panes: Vec<EditorPane>,
    pub active_pane: usize,
    
    pub recent_files: Vec<PathBuf>,
    
    next_buffer_id: usize,
}

impl WorkspaceState {
    pub fn new(project_root: Option<PathBuf>) -> Self {
        Self {
            project_root,
            documents: HashMap::new(),
            buffers: HashMap::new(),
            panes: Vec::new(),
            active_pane: 0,
            recent_files: Vec::new(),
            next_buffer_id: 1,
        }
    }

    pub fn add_buffer(&mut self, document: Document, buffer: TextBuffer) -> BufferId {
        let id = BufferId::new(self.next_buffer_id);
        self.next_buffer_id += 1;
        
        self.documents.insert(id, document);
        self.buffers.insert(id, buffer);
        
        id
    }
    
    pub fn open_pane(&mut self, buffer_id: BufferId) {
        self.panes.push(EditorPane::new(buffer_id));
        self.active_pane = self.panes.len() - 1;
    }

    pub fn active_pane_mut(&mut self) -> Option<&mut EditorPane> {
        self.panes.get_mut(self.active_pane)
    }

    pub fn active_pane_ref(&self) -> Option<&EditorPane> {
        self.panes.get(self.active_pane)
    }

    pub fn active_buffer_mut(&mut self) -> Option<&mut TextBuffer> {
        let buffer_id = self.active_pane_ref().map(|p| p.buffer_id)?;
        self.buffers.get_mut(&buffer_id)
    }

    pub fn active_document_mut(&mut self) -> Option<&mut Document> {
        let buffer_id = self.active_pane_ref().map(|p| p.buffer_id)?;
        self.documents.get_mut(&buffer_id)
    }
}
