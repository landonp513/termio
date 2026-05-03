use std::{fs, path::PathBuf};

use anyhow::{Result, anyhow};
use crossterm::{
    event::{Event, read},
    terminal::size,
};

use crate::RawModeGuard;

#[derive(Default, Clone)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

pub struct Snapshot {
    pub buffer: Vec<String>,
    pub cursor: Position,
}

pub struct PendingOp {
    pub operator: Operator,
}

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Command,
    Search,
    Visual,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub enum Operator {
    Delete,
    //Change,
    //Yank,
}

pub struct Editor {
    pub buffer: Vec<String>,
    pub cursor: Position,
    pub row_offset: usize,
    pub filename: Option<PathBuf>,
    pub mode: Mode,
    pub status: String,
    pub command_buffer: String,
    pub undo_stack: Vec<Snapshot>,
    pub redo_stack: Vec<Snapshot>,
    pub search_query: String,
    pub last_search: String,
    pub pending_op: Option<PendingOp>,
    pub selection_anchor: Option<Position>,
}
impl Editor {
    pub fn new() -> Self {
        Editor {
            buffer: vec![String::new()],
            cursor: Position::default(),
            row_offset: 0,
            filename: None,
            mode: Mode::default(),
            status: String::from(""),
            command_buffer: String::from(""),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            search_query: String::from(""),
            last_search: String::from(""),
            pending_op: None,
            selection_anchor: None,
        }
    }
    pub fn assert_invariants(&self) {
        debug_assert!(!self.buffer.is_empty(), "buffer must always have >= 1 line");
        debug_assert!(
            self.cursor.row < self.buffer.len(),
            "cursor row {} out of bounds (buffer len {})",
            self.cursor.row,
            self.buffer.len()
        );
        let line_len = self.buffer[self.cursor.row].chars().count();
        debug_assert!(
            self.cursor.col <= line_len,
            "cursor col {} out of bounds for line of length {}",
            self.cursor.col,
            line_len
        );
    }
    pub fn prompt_state(&self) -> Option<(char, &str)> {
        match self.mode {
            Mode::Command => Some((':', &self.command_buffer)),
            Mode::Search => Some(('/', &self.search_query)),
            _ => None,
        }
    }
    pub fn open(&mut self, path: PathBuf) -> Result<()> {
        let contents = fs::read_to_string(&path)?;
        self.buffer = contents.lines().map(String::from).collect();
        self.filename = Some(path);
        self.assert_invariants();
        Ok(())
    }
    pub fn save(&mut self) -> Result<()> {
        let path = self.filename.as_ref().ok_or_else(|| anyhow!("no filename"))?;
        let contents = self.buffer.join("\n");
        fs::write(path, &contents)?;
        self.status = format!("saved {} bytes", contents.len());
        Ok(())
    }
    pub fn scroll(&mut self) {
        let (_cols, rows) = size().unwrap_or((80, 24));
        let rows = (rows as usize).saturating_sub(1);
        if self.cursor.row < self.row_offset {
            self.row_offset = self.cursor.row;
        } else if self.cursor.row >= self.row_offset + rows {
            self.row_offset = self.cursor.row - rows + 1;
        }
    }
    pub fn run(&mut self) -> Result<()> {
        let _guard = RawModeGuard::new()?;
        loop {
            self.render()?;
            match read()? {
                Event::Key(event) => {
                    let keep_running = match self.mode {
                        Mode::Normal => self.handle_normal(event)?,
                        Mode::Insert => self.handle_insert(event)?,
                        Mode::Command => self.handle_command(event)?,
                        Mode::Search => self.handle_search(event)?,
                        Mode::Visual => self.handle_visual(event)?,
                    };
                    if !keep_running {
                        return Ok(());
                    }
                },
                _ => {},
            }
        }
    }
}
