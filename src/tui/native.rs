use std::io::{self, Write};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use super::*;

/// A wrapper to handle writing to crossterm's alternate screen mode
pub struct TuiWriter {
    stdout: io::Stdout,
}

impl TuiWriter {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode().map(|_| Self {
            stdout: io::stdout(),
        })
    }
}

impl Write for TuiWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}

impl Drop for TuiWriter {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.stdout, LeaveAlternateScreen);
    }
}

impl Tui<CrosstermBackend<TuiWriter>> {
    pub fn new() -> io::Result<Self> {
        let mut out = TuiWriter::new()?;
        execute!(out, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(out);

        Terminal::new(backend).map(Self)
    }
}
