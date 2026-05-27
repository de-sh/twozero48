use std::{
    collections::HashSet,
    io::{self, Write},
};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{Play, Tile};

const CELL_W: u16 = 10;
const CELL_H: u16 = 5;
const HEADER_H: u16 = 2;
const FOOTER_H: u16 = 1;
const EMPTY_BG: Color = Color::Rgb(40, 40, 40);

fn brighten(color: Color, amt: u8) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            r.saturating_add(amt),
            g.saturating_add(amt),
            b.saturating_add(amt),
        ),
        c => c,
    }
}

/// Returns the [`Color`] of the tile(used for rendering)
fn tile_color(tile: Tile) -> Color {
    match tile.exponent() {
        0 => Color::Rgb(180, 180, 180),
        1 => Color::Rgb(255, 220, 80),
        2 => Color::Rgb(255, 165, 30),
        3 => Color::Rgb(255, 100, 20),
        4 => Color::Rgb(240, 50, 50),
        5 => Color::Rgb(200, 20, 120),
        6 => Color::Rgb(150, 0, 200),
        7 => Color::Rgb(80, 20, 220),
        8 => Color::Rgb(20, 100, 255),
        9 => Color::Rgb(0, 200, 220),
        10 => Color::Rgb(20, 220, 120),
        11 => Color::Rgb(255, 215, 0),
        12 => Color::Rgb(255, 255, 255),
        _ => Color::Rgb(200, 200, 200),
    }
}

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

pub(crate) struct Tui<B: Backend>(Terminal<B>);

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

    pub fn render_board(
        &mut self,
        game: &impl Play,
        message: Option<&str>,
        x_shift: i16,
        y_shift: i16,
        flash: &HashSet<(usize, usize)>,
    ) -> io::Result<()> {
        self.0
            .draw(|f| -> () {
                let board = game.board();
                let area = f.area();
                let board_size = game.board().size() as u16;
                let board_w = CELL_W * board_size;
                let board_h = CELL_H * board_size;

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(HEADER_H),
                        Constraint::Min(board_h),
                        Constraint::Length(FOOTER_H),
                    ])
                    .split(area);

                let header_area = chunks[0];
                let board_area = chunks[1];
                let footer_area = chunks[2];

                let title = Paragraph::new(Line::from(vec![
                    Span::styled(
                        "2",
                        Style::default()
                            .fg(Color::Rgb(255, 100, 20))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        "0",
                        Style::default()
                            .fg(Color::Rgb(255, 150, 20))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        "4",
                        Style::default()
                            .fg(Color::Rgb(200, 20, 120))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        "8",
                        Style::default()
                            .fg(Color::Rgb(80, 20, 220))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]))
                .alignment(Alignment::Center);
                f.render_widget(
                    title,
                    Rect::new(header_area.x, header_area.y, header_area.width, 1),
                );

                let score_line = Paragraph::new(Line::from(vec![
                    Span::styled("SCORE  ", Style::default().fg(Color::Rgb(100, 100, 100))),
                    Span::styled(
                        game.score().to_string(),
                        Style::default()
                            .fg(Color::Rgb(230, 230, 230))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]))
                .alignment(Alignment::Center);
                f.render_widget(
                    score_line,
                    Rect::new(header_area.x, header_area.y + 1, header_area.width, 1),
                );

                let x_base =
                    board_area.x as i16 + board_area.width.saturating_sub(board_w) as i16 / 2;
                let y_base =
                    board_area.y as i16 + board_area.height.saturating_sub(board_h) as i16 / 2;

                let board_size = board.size();
                for row_i in 0..board_size {
                    for col_i in 0..board_size {
                        let tile = board[(row_i, col_i)];
                        let cx = x_base + col_i as i16 * CELL_W as i16 + x_shift;
                        let cy = y_base + row_i as i16 * CELL_H as i16 + y_shift;
                        if cx < 0 || cy < 0 {
                            continue;
                        }
                        let cx = cx as u16;
                        let cy = cy as u16;
                        if cx + CELL_W > area.width || cy + CELL_H > area.height {
                            continue;
                        }

                        let cell_area = Rect::new(cx, cy, CELL_W, CELL_H);
                        let is_empty = tile == Tile::EMPTY;
                        let is_flash = flash.contains(&(row_i, col_i)) && !is_empty;

                        let (fg, bg) = if is_empty {
                            (EMPTY_BG, EMPTY_BG)
                        } else {
                            let base = tile_color(tile);
                            let bg = if is_flash { brighten(base, 70) } else { base };
                            (Color::White, bg)
                        };

                        let block = Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .style(Style::default().bg(bg));

                        let inner = block.inner(cell_area);
                        f.render_widget(block, cell_area);

                        if !is_empty {
                            let text = Paragraph::new(Span::styled(
                                tile.to_string(),
                                Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
                            ))
                            .alignment(Alignment::Center);
                            let mid_y = inner.y + inner.height / 2;
                            f.render_widget(text, Rect::new(inner.x, mid_y, inner.width, 1));
                        }
                    }
                }

                let footer_line = if let Some(msg) = message {
                    Line::from(Span::styled(
                        msg,
                        Style::default()
                            .fg(Color::Rgb(255, 80, 80))
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(vec![
                        Span::styled(
                            "WASD",
                            Style::default()
                                .fg(Color::Rgb(200, 200, 200))
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" / arrows: move  ·  "),
                        Span::styled(
                            "Q",
                            Style::default()
                                .fg(Color::Rgb(200, 200, 200))
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(": quit  ·  Win: "),
                        Span::styled(
                            game.winning().to_string(),
                            Style::default()
                                .fg(tile_color(game.winning()))
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])
                };
                f.render_widget(
                    Paragraph::new(footer_line).alignment(Alignment::Center),
                    footer_area,
                );
            })
            .map_err(|e| io::Error::other(e.to_string()))?;
        Ok(())
    }
}

impl<B: Backend> Drop for Tui<B> {
    fn drop(&mut self) {
        let _ = self.0.show_cursor();
    }
}
