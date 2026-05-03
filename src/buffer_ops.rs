use crate::{
    editor::{Direction, Editor, Mode, Position},
    text::char_to_byte,
};

impl Editor {
    pub fn move_cursor(&mut self, dir: Direction) {
        let row = self.cursor.row;
        match dir {
            Direction::Up => self.cursor.row = row.saturating_sub(1),
            Direction::Down => {
                if row + 1 < self.buffer.len() {
                    self.cursor.row = row + 1;
                }
            },
            Direction::Left => self.cursor.col = self.cursor.col.saturating_sub(1),
            Direction::Right => {
                let line_len = self.buffer[self.cursor.row].chars().count();
                if self.cursor.col < line_len {
                    self.cursor.col += 1;
                }
            },
        }
        let line_len = self.buffer[self.cursor.row].chars().count();
        if self.cursor.col > line_len {
            self.cursor.col = line_len;
        }
        self.assert_invariants();
    }
    fn next_char_pos(&self, row: usize, col: usize) -> (usize, usize) {
        let line_len = self.buffer[row].chars().count();
        if col + 1 <= line_len {
            (row, col + 1)
        } else if row + 1 < self.buffer.len() {
            (row + 1, 0)
        } else {
            (row, col)
        }
    }
    fn prev_char_pos(&self, row: usize, col: usize) -> (usize, usize) {
        if col > 0 {
            (row, col - 1)
        } else if row > 0 {
            let prev_line_len = self.buffer[row - 1].chars().count();
            (row - 1, prev_line_len)
        } else {
            (row, col)
        }
    }
    fn is_at_whitespace(&self, row: usize, col: usize) -> bool {
        let line = &self.buffer[row];
        let chars: Vec<char> = line.chars().collect();
        if col >= chars.len() {
            return true;
        }
        chars[col].is_whitespace()
    }
    fn at_start(&self, row: usize, col: usize) -> bool {
        row == 0 && col == 0
    }
    fn at_end(&self, row: usize, col: usize) -> bool {
        row + 1 == self.buffer.len() && col >= self.buffer[row].chars().count()
    }
    pub fn word_forward_pos(&self) -> Position {
        let (mut row, mut col) = (self.cursor.row, self.cursor.col);
        while !self.is_at_whitespace(row, col) && !self.at_end(row, col) {
            (row, col) = self.next_char_pos(row, col);
        }
        while self.is_at_whitespace(row, col) && !self.at_end(row, col) {
            (row, col) = self.next_char_pos(row, col);
        }
        Position { row, col }
    }
    pub fn word_backward_pos(&self) -> Position {
        let (mut row, mut col) = (self.cursor.row, self.cursor.col);
        while !self.is_at_whitespace(row, col) && !self.at_start(row, col) {
            (row, col) = self.prev_char_pos(row, col);
        }
        while self.is_at_whitespace(row, col) && !self.at_start(row, col) {
            (row, col) = self.prev_char_pos(row, col);
        }
        Position { row, col }
    }
    pub fn move_word_forward(&mut self) {
        let target = self.word_forward_pos();
        self.cursor = target;
        self.assert_invariants();
    }
    pub fn move_word_backward(&mut self) {
        let target = self.word_backward_pos();
        self.cursor = target;
        self.assert_invariants();
    }
    pub fn insert_char(&mut self, c: char) {
        let line = &mut self.buffer[self.cursor.row];
        let byte_idx = char_to_byte(line, self.cursor.col);
        line.insert(byte_idx, c);
        self.cursor.col += 1;
        self.assert_invariants();
    }
    pub fn delete_char_before(&mut self) {
        if self.cursor.col > 0 {
            let line = &mut self.buffer[self.cursor.row];
            let byte_idx = char_to_byte(line, self.cursor.col - 1);
            line.remove(byte_idx);
            self.cursor.col -= 1;
        } else if self.cursor.row > 0 {
            let current = self.buffer.remove(self.cursor.row);
            let prev_len = self.buffer[self.cursor.row - 1].chars().count();
            self.buffer[self.cursor.row - 1].push_str(&current);
            self.cursor.row -= 1;
            self.cursor.col = prev_len;
        }
        self.assert_invariants();
    }
    pub fn delete_char_at(&mut self) {
        let line = &self.buffer[self.cursor.row];
        let line_len = line.chars().count();

        if self.cursor.col >= line_len {
            return;
        }

        self.push_undo();

        let line = &mut self.buffer[self.cursor.row];
        let byte_idx = char_to_byte(line, self.cursor.col);
        line.remove(byte_idx);

        let new_len = line.chars().count();
        if self.cursor.col >= new_len && new_len > 0 {
            self.cursor.col = new_len - 1;
        } else if new_len == 0 {
            self.cursor.col = 0;
        }

        self.assert_invariants();
    }
    pub fn delete_line(&mut self) {
        self.push_undo();
        if self.buffer.len() == 1 {
            self.buffer[0].clear();
        } else {
            self.buffer.remove(self.cursor.row);
            if self.cursor.row >= self.buffer.len() {
                self.cursor.row = self.buffer.len() - 1;
            }
        }
        let line_len = self.buffer[self.cursor.row].chars().count();
        if self.cursor.col > line_len {
            self.cursor.col = line_len;
        }
        self.assert_invariants();
    }
    pub fn delete_range(&mut self, start: Position, end: Position) {
        if start.row != end.row {
            return;
        }
        let (lo, hi) = if start.col <= end.col {
            (start.col, end.col)
        } else {
            (end.col, start.col)
        };
        self.push_undo();
        let line = &mut self.buffer[start.row];
        let lo_byte = char_to_byte(line, lo);
        let hi_byte = char_to_byte(line, hi);
        line.replace_range(lo_byte..hi_byte, "");
        self.cursor.col = lo;
        self.assert_invariants();
    }
    pub fn insert_newline(&mut self) {
        let line = &mut self.buffer[self.cursor.row];
        let byte_idx = char_to_byte(line, self.cursor.col);
        let tail = line.split_off(byte_idx);
        self.buffer.insert(self.cursor.row + 1, tail);
        self.cursor.row += 1;
        self.cursor.col = 0;
        self.assert_invariants();
    }
    pub fn open_line_below(&mut self) {
        self.push_undo();
        self.buffer.insert(self.cursor.row + 1, String::new());
        self.cursor.row += 1;
        self.cursor.col = 0;
        self.mode = Mode::Insert;
        self.assert_invariants();
    }
    pub fn open_line_above(&mut self) {
        self.push_undo();
        self.buffer.insert(self.cursor.row, String::new());
        self.cursor.col = 0;
        self.mode = Mode::Insert;
        self.assert_invariants();
    }
    pub fn selection_range(&self) -> Option<(Position, Position)> {
        let anchor = self.selection_anchor.as_ref()?;;
        let cursor = &self.cursor;
        let (start, end) = if (anchor.row, anchor.col) <= (cursor.row, cursor.col) {
            (anchor.clone(), cursor.clone())
        } else {
            (cursor.clone(), anchor.clone())
        };
        Some((start, end))
    }
}
