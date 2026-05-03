mod buffer_ops;
mod editor;
mod handlers;
mod history;
mod render;
mod search;
mod text;

use std::{env::args, io, path::PathBuf};

use anyhow::Result;
use crossterm::{
    execute, terminal,
    terminal::{disable_raw_mode, enable_raw_mode},
};
pub use editor::Editor;

pub struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;
        Ok(Self)
    }
}
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), terminal::LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

fn main() -> Result<()> {
    let mut editor = Editor::new();
    if let Some(arg) = args().nth(1) {
        editor.open(PathBuf::from(arg))?;
    }
    editor.run()?;
    Ok(())
}
