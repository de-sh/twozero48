use std::io;

use rand::Rng;
use ratatui::{Terminal, backend::Backend};

use super::Screen;

pub struct Tui<B: Backend>(pub(super) Terminal<B>);

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
