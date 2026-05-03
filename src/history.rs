use crate::editor::{Editor, Snapshot};

impl Editor {
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            buffer: self.buffer.clone(),
            cursor: self.cursor.clone(),
        }
    }
    pub fn push_undo(&mut self) {
        self.undo_stack.push(self.snapshot());
        self.redo_stack.clear();
    }
    pub fn undo(&mut self) {
        if let Some(snap) = self.undo_stack.pop() {
            let current = self.snapshot();
            self.buffer = snap.buffer;
            self.cursor = snap.cursor;
            self.redo_stack.push(current);
        } else {
            self.status = "already at oldest change".to_string();
        }
        self.assert_invariants();
    }
    pub fn redo(&mut self) {
        if let Some(snap) = self.redo_stack.pop() {
            let current = self.snapshot();
            self.buffer = snap.buffer;
            self.cursor = snap.cursor;
            self.undo_stack.push(current);
        }
        self.assert_invariants();
    }
}
