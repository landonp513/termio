use std::{
    io::{self, Write},
    write,
};

use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    queue,
    terminal::{Clear, ClearType, size},
};

use crate::editor::{Editor, Mode};

impl Editor {
    pub fn render(&mut self) -> Result<()> {
        self.assert_invariants();
        self.scroll();
        let mut stdout = io::stdout();
        let (cols, rows) = size()?;
        let cols = cols as usize;
        let rows = rows as usize;
        queue!(
            stdout,
            crossterm::cursor::Hide,
            Clear(ClearType::All),
            MoveTo(0, 0),
        )?;

        for i in 0..(rows as usize).saturating_sub(1) {
            queue!(stdout, MoveTo(0, i as u16))?;
            let buf_row = self.row_offset + i;
            if buf_row < self.buffer.len() {
                let line = &self.buffer[buf_row];
                let visible = line.chars().take(cols as usize).collect::<String>();
                write!(stdout, "{}", visible)?;
            } else {
                write!(stdout, "~")?;
            }
        }
        queue!(stdout, MoveTo(0, (rows - 1) as u16))?;
        let filename = self.filename.as_ref().and_then(|p| p.to_str()).unwrap_or("[No Name]");
        let right = self.status.clone();
        let right_width = right.chars().count();
        let left_max = cols.saturating_sub(right_width + 1);
        let left_full = match self.mode {
            Mode::Command => format!(":{}", self.command_buffer),
            Mode::Search => format!("/{}", self.search_query),
            _ => format!(
                "{:?} | {} | {}:{}",
                self.mode,
                filename,
                self.cursor.row + 1,
                self.cursor.col + 1
            ),
        };
        let left = left_full.chars().take(left_max).collect::<String>();
        let padding = cols - left.chars().count() - right_width;
        let status = format!("{}{}{}", left, " ".repeat(padding), right);
        let visible = status.chars().take(cols as usize).collect::<String>();
        write!(stdout, "{}", visible)?;
        let cursor_x;
        let cursor_y;
        if let Some((_prefix, content)) = self.prompt_state() {
            cursor_x = (1 + content.chars().count()) as u16;
            cursor_y = (rows - 1) as u16;
        } else {
            cursor_x = self.cursor.col as u16;
            cursor_y = self.cursor.row.saturating_sub(self.row_offset) as u16;
        }

        queue!(stdout, MoveTo(cursor_x, cursor_y), crossterm::cursor::Show)?;
        stdout.flush()?;
        Ok(())
    }
}
