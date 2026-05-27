use std::io::{self, Write};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::Rng;
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};

use super::Screen;

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

pub(crate) struct Tui<B: Backend>(pub(super) Terminal<B>);

impl Tui<CrosstermBackend<TuiWriter>> {
    pub fn new() -> io::Result<Self> {
        let mut out = TuiWriter::new()?;
        execute!(out, EnterAlternateScreen)?;
        Terminal::new(CrosstermBackend::new(out)).map(Self)
    }
}

impl<B: Backend> Tui<B> {
    pub fn from_backend(backend: B) -> Result<Self, B::Error> {
        Terminal::new(backend).map(Self)
    }

    pub fn draw<R: Rng>(&mut self, screen: Screen<'_, R>) -> io::Result<()> {
        self.0
            .draw(|frame| screen.render(frame))
            .map_err(|e| io::Error::other(e.to_string()))
            .map(|_| ())
    }
}

impl<B: Backend> Drop for Tui<B> {
    fn drop(&mut self) {
        let _ = self.0.show_cursor();
    }
}
