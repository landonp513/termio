use crate::{
    editor::Editor,
    text::{byte_offset_search, byte_to_char, char_to_byte},
};

impl Editor {
    pub fn find_next(&mut self) {
        if self.last_search.is_empty() {
            return;
        }
        let query = &self.last_search;
        let start_row = self.cursor.row;
        let start_col = self.cursor.col;

        let line = &self.buffer[start_row];
        if let Some(byte_idx) = byte_offset_search(line, start_col + 1, query) {
            self.cursor.col = byte_to_char(line, byte_idx);
            return;
        }

        for row in (start_row + 1)..self.buffer.len() {
            if let Some(byte_idx) = self.buffer[row].find(query) {
                self.cursor.row = row;
                self.cursor.col = byte_to_char(&self.buffer[row], byte_idx);
                return;
            }
        }

        for row in 0..=start_row {
            let line = &self.buffer[row];
            let end_byte = if row == start_row {
                char_to_byte(line, start_col)
            } else {
                line.len()
            };
            if let Some(byte_idx) = line[..end_byte].find(query) {
                self.cursor.row = row;
                self.cursor.col = byte_to_char(line, byte_idx);
                self.status = format!("search wrapped");
                return;
            }
        }

        self.status = format!("not found: {}", query);
    }
    pub fn find_prev(&mut self) {
        if self.last_search.is_empty() {
            return;
        }
        let query = self.last_search.clone();
        let start_row = self.cursor.row;
        let start_col = self.cursor.col;

        let line = &self.buffer[start_row];
        let end_byte = char_to_byte(line, start_col);
        if let Some(byte_idx) = line[..end_byte].rfind(&query) {
            self.cursor.col = byte_to_char(line, byte_idx);
            return;
        }

        for row in (0..start_row).rev() {
            if let Some(byte_idx) = self.buffer[row].rfind(&query) {
                self.cursor.row = row;
                self.cursor.col = byte_to_char(&self.buffer[row], byte_idx);
                return;
            }
        }

        for row in (start_row..self.buffer.len()).rev() {
            let line = &self.buffer[row];
            let search_slice = if row == start_row {
                let start_byte = char_to_byte(line, start_col + 1);
                if start_byte >= line.len() {
                    continue;
                }
                &line[start_byte..]
            } else {
                &line[..]
            };
            if let Some(rel) = search_slice.rfind(&query) {
                let byte_idx = if row == start_row {
                    rel + char_to_byte(line, start_col + 1)
                } else {
                    rel
                };
                self.cursor.row = row;
                self.cursor.col = byte_to_char(line, byte_idx);
                self.status = "search wrapped".to_string();
                return;
            }
        }

        self.status = format!("not found: {}", query);
    }
}
