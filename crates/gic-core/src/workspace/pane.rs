use crate::{BufferId, CursorPosition};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Vertical,
    Horizontal,
}

/// Represents a single view into a buffer.
/// A pane has its own cursor position and viewport state, allowing multiple panes
/// to view the same buffer independently.
#[derive(Debug, Clone)]
pub struct EditorPane {
    pub buffer_id: BufferId,
    pub cursor: CursorPosition,
    pub scroll_row: usize,
    pub scroll_col: usize,
}

impl EditorPane {
    pub fn new(buffer_id: BufferId) -> Self {
        Self {
            buffer_id,
            cursor: CursorPosition::zero(),
            scroll_row: 0,
            scroll_col: 0,
        }
    }
}
