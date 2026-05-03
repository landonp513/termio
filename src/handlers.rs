use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor::{Direction, Editor, Mode, Operator, PendingOp, Position};

impl Editor {
    pub fn handle_normal(&mut self, event: KeyEvent) -> Result<bool> {
        self.status.clear();
        if let Some(pending) = self.pending_op.take() {
            return self.handle_op_motion(pending, event);
        }
        match (event.modifiers, event.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                return Ok(false);
            },
            (_, KeyCode::Char('h')) => self.move_cursor(Direction::Left),
            (_, KeyCode::Char('j')) => self.move_cursor(Direction::Down),
            (_, KeyCode::Char('k')) => self.move_cursor(Direction::Up),
            (_, KeyCode::Char('l')) => self.move_cursor(Direction::Right),
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                if let Err(e) = self.save() {
                    self.status = format!("error: {}", e);
                }
            },
            (_, KeyCode::Char('i')) => {
                self.push_undo();
                self.mode = Mode::Insert;
            },
            (_, KeyCode::Char(':')) => {
                self.mode = Mode::Command;
                self.command_buffer.clear();
            },
            (_, KeyCode::Char('u')) => self.undo(),
            (KeyModifiers::CONTROL, KeyCode::Char('r')) => self.redo(),
            (_, KeyCode::Char('/')) => {
                self.mode = Mode::Search;
                self.search_query.clear();
            },
            (_, KeyCode::Char('n')) => self.find_next(),
            (_, KeyCode::Char('N')) => self.find_prev(),
            (_, KeyCode::Char('x')) => self.delete_char_at(),
            (_, KeyCode::Char('o')) => self.open_line_below(),
            (_, KeyCode::Char('O')) => self.open_line_above(),
            (_, KeyCode::Char('d')) => {
                self.status = "d op".to_string();
                self.pending_op = Some(PendingOp { operator: crate::editor::Operator::Delete})
            },
            (_, KeyCode::Char('w')) => self.move_word_forward(),
            (_, KeyCode::Char('b')) => self.move_word_backward(),
            (_, KeyCode::Char('0')) => {
                self.cursor.col = 0;
            },
            (_, KeyCode::Char('$')) => {
                let line_len = self.buffer[self.cursor.row].chars().count();
                self.cursor.col = line_len;
            }
            (_, KeyCode::Char('v')) => {
                self.selection_anchor = Some(self.cursor.clone());
                self.mode = Mode::Visual;
            }
            _ => {},
        }
        Ok(true)
    }
    pub fn handle_insert(&mut self, event: KeyEvent) -> Result<bool> {
        self.status.clear();
        match (event.modifiers, event.code) {
            (_, KeyCode::Esc) => self.mode = Mode::Normal,
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => self.insert_char(c),
            (_, KeyCode::Backspace) => self.delete_char_before(),
            (_, KeyCode::Enter) => self.insert_newline(),
            (_, KeyCode::Left) => self.move_cursor(Direction::Left),
            (_, KeyCode::Down) => self.move_cursor(Direction::Down),
            (_, KeyCode::Up) => self.move_cursor(Direction::Up),
            (_, KeyCode::Right) => self.move_cursor(Direction::Right),
            _ => {},
        }
        Ok(true)
    }
    pub fn handle_command(&mut self, event: KeyEvent) -> Result<bool> {
        match (event.modifiers, event.code) {
            (_, KeyCode::Esc) => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            },
            (_, KeyCode::Char(c)) => self.command_buffer.push(c),
            (_, KeyCode::Backspace) => {
                if self.command_buffer.is_empty() {
                    self.mode = Mode::Normal;
                } else {
                    self.command_buffer.pop();
                }
            },
            (_, KeyCode::Enter) => {
                let cmd = std::mem::take(&mut self.command_buffer);
                self.mode = Mode::Normal;
                return self.execute_command(&cmd);
            },
            _ => {},
        }
        Ok(true)
    }
    fn execute_command(&mut self, cmd: &str) -> Result<bool> {
        match cmd {
            "w" => {
                self.save()?;
            },
            "q" => return Ok(false),
            "wq" => {
                self.save()?;
                return Ok(false);
            },
            "q!" => return Ok(false),
            _ => {
                self.status = format!("unknown command: {}", cmd);
            },
        }
        Ok(true)
    }
    fn handle_op_motion(&mut self, pending: PendingOp, event: KeyEvent) -> Result<bool> {
        match pending.operator {
            Operator::Delete => {
                if event.code == KeyCode::Char('d') {
                    self.delete_line();
                    return Ok(true);
                }
                if let Some((start, end)) = self.motion_range(event) {
                    self.delete_range(start, end);
                }
            }
        }
        Ok(true)
    }
    fn motion_range(&self, event: KeyEvent) -> Option<(Position, Position)> {
        let start = self.cursor.clone();
        let end = match (event.modifiers, event.code) {
            (_, KeyCode::Char('w')) => self.word_forward_pos(),
            (_, KeyCode::Char('b')) => self.word_backward_pos(),
            (_, KeyCode::Char('h')) | (_, KeyCode::Left) => {
                Position { row: start.row, col: start.col.saturating_sub(1) }
            }
            (_, KeyCode::Char('l')) | (_, KeyCode::Right) => {
                let line_len = self.buffer[start.row].chars().count();
                Position { row: start.row, col: (start.col + 1).min(line_len) }
            }
            _ => return None,
        };
        Some((start, end))
    }
    pub fn handle_search(&mut self, event: KeyEvent) -> Result<bool> {
        match (event.modifiers, event.code) {
            (_, KeyCode::Esc) => {
                self.mode = Mode::Normal;
                self.search_query.clear();
            },
            (_, KeyCode::Char(c)) => self.search_query.push(c),
            (_, KeyCode::Backspace) => {
                if self.search_query.is_empty() {
                    self.mode = Mode::Normal;
                } else {
                    self.search_query.pop();
                }
            },
            (_, KeyCode::Enter) => {
                let query = std::mem::take(&mut self.search_query);
                self.last_search = query;
                self.mode = Mode::Normal;
                self.find_next();
            },
            _ => {},
        }
        Ok(true)
    }
    pub fn handle_visual(&mut self, event: KeyEvent) -> Result<bool> {
        self.status.clear();
        match (event.modifiers, event.code) {
            (_, KeyCode::Esc) | (_, KeyCode::Char('v')) => {
                self.selection_anchor = None;
                self.mode = Mode::Normal;
            }
            (_, KeyCode::Char('h')) => self.move_cursor(Direction::Left),
            (_, KeyCode::Char('j')) => self.move_cursor(Direction::Down),
            (_, KeyCode::Char('k')) => self.move_cursor(Direction::Up),
            (_, KeyCode::Char('l')) => self.move_cursor(Direction::Right),
            (_, KeyCode::Char('w')) => self.move_word_forward(),
            (_, KeyCode::Char('b')) => self.move_word_backward(),
            (_, KeyCode::Char('d')) => self.delete_selection(),
            _ => {},
        }
        Ok(true)
    }
}
