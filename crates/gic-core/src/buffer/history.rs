use crate::buffer::commands::BufferCommand;
use crate::buffer::cursor::CursorPosition;
use crate::buffer::selection::Selection;
use serde::{Deserialize, Serialize};

/// Maximum default capacity for undo history stack.
pub const DEFAULT_MAX_UNDO_LEVELS: usize = 1000;

/// Atomic transaction group of buffer commands representing a single logical edit operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandGroup {
    /// Inverted list of commands in order of execution.
    pub commands: Vec<BufferCommand>,
    /// Cursor position prior to executing group commands.
    pub cursor_before: CursorPosition,
    /// Cursor position after executing group commands.
    pub cursor_after: CursorPosition,
    /// Selection state prior to edit.
    pub selection_before: Option<Selection>,
    /// Selection state after edit.
    pub selection_after: Option<Selection>,
}

impl CommandGroup {
    /// Creates a new `CommandGroup`.
    pub fn new(cursor_before: CursorPosition, selection_before: Option<Selection>) -> Self {
        Self {
            commands: Vec::new(),
            cursor_before,
            cursor_after: cursor_before,
            selection_before,
            selection_after: None,
        }
    }

    /// Adds a command to current transaction group.
    pub fn add_command(&mut self, command: BufferCommand) {
        self.commands.push(command);
    }

    /// Finalizes cursor and selection state after group execution.
    pub fn finalize(&mut self, cursor_after: CursorPosition, selection_after: Option<Selection>) {
        self.cursor_after = cursor_after;
        self.selection_after = selection_after;
    }
}

/// History stack managing Undo and Redo operations using command transactions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoRedoHistory {
    undo_stack: Vec<CommandGroup>,
    redo_stack: Vec<CommandGroup>,
    max_undo_levels: usize,
}

impl Default for UndoRedoHistory {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_UNDO_LEVELS)
    }
}

impl UndoRedoHistory {
    /// Creates a new `UndoRedoHistory` with specified max capacity.
    pub fn new(max_undo_levels: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_levels: if max_undo_levels == 0 {
                1
            } else {
                max_undo_levels
            },
        }
    }

    /// Pushes a completed transaction group onto the undo stack and clears redo stack.
    pub fn push_group(&mut self, group: CommandGroup) {
        if group.commands.is_empty() {
            return;
        }

        self.undo_stack.push(group);
        self.redo_stack.clear();

        if self.undo_stack.len() > self.max_undo_levels {
            self.undo_stack.remove(0); // Evict oldest
        }
    }

    /// Pops the last transaction group from undo stack and pushes to redo stack.
    pub fn pop_undo(&mut self) -> Option<CommandGroup> {
        let group = self.undo_stack.pop()?;
        self.redo_stack.push(group.clone());
        Some(group)
    }

    /// Pops the last transaction group from redo stack and pushes to undo stack.
    pub fn pop_redo(&mut self) -> Option<CommandGroup> {
        let group = self.redo_stack.pop()?;
        self.undo_stack.push(group.clone());
        Some(group)
    }

    /// Clears both undo and redo stacks.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Returns true if undo stack is empty.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns true if redo stack is empty.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Returns length of undo stack.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Returns length of redo stack.
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_redo_history_push_pop() {
        let mut history = UndoRedoHistory::new(5);
        assert!(!history.can_undo());
        assert!(!history.can_redo());

        let mut group = CommandGroup::new(CursorPosition::zero(), None);
        group.add_command(BufferCommand::InsertChar {
            pos: CursorPosition::zero(),
            ch: 'X',
        });
        group.finalize(CursorPosition::new(0, 1), None);

        history.push_group(group);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        let popped_undo = history.pop_undo();
        assert!(popped_undo.is_some());
        assert!(!history.can_undo());
        assert!(history.can_redo());

        let popped_redo = history.pop_redo();
        assert!(popped_redo.is_some());
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }
}
