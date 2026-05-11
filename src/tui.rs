use std::{
    collections::HashSet,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use twozero48::{Game, Move, Tile};

const CELL_W: u16 = 10;
const CELL_H: u16 = 5;
const HEADER_H: u16 = 2;
const FOOTER_H: u16 = 1;
const EMPTY_BG: Color = Color::Rgb(40, 40, 40);
const FLASH_DURATION: Duration = Duration::from_millis(120);

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

struct AnimState {
    dx: i16,
    dy: i16,
    started: Instant,
}

impl AnimState {
    fn new(mov: Move) -> Option<Self> {
        let (dx, dy) = match mov {
            Move::Left => (-2, 0),
            Move::Right => (2, 0),
            Move::Up => (0, -2),
            Move::Down => (0, 2),
            Move::Dont => return None,
        };

        Some(Self {
            dx,
            dy,
            started: Instant::now(),
        })
    }

    fn shift(&self) -> (i16, i16) {
        let elapsed = self.started.elapsed().as_millis().min(i16::MAX as u128) as i16;
        let step = (elapsed / 65).min(3);
        let remaining = (3 - step).max(0);
        (self.dx * remaining * 2 / 3, self.dy * remaining * 2 / 3)
    }

    fn expired(&self) -> bool {
        self.started.elapsed() >= Duration::from_millis(200)
    }
}

pub struct MoveEffects {
    anim: Option<AnimState>,
    flash: HashSet<(usize, usize)>,
    flash_until: Option<Instant>,
}

impl MoveEffects {
    pub fn new() -> Self {
        Self {
            anim: None,
            flash: HashSet::new(),
            flash_until: None,
        }
    }

    pub fn record_move(&mut self, mov: Move, old_board: &[Vec<Tile>], new_board: &[Vec<Tile>]) {
        self.anim = AnimState::new(mov);
        self.flash = changed_cells(old_board, new_board);
        self.flash_until = Some(Instant::now() + FLASH_DURATION);
    }

    pub fn tick(&mut self) {
        if self
            .flash_until
            .map(|t| Instant::now() >= t)
            .unwrap_or(false)
        {
            self.flash.clear();
            self.flash_until = None;
        }

        if self.anim.as_ref().map(|a| a.expired()).unwrap_or(false) {
            self.anim = None;
        }
    }

    pub fn shift(&self) -> (i16, i16) {
        self.anim.as_ref().map(|a| a.shift()).unwrap_or((0, 0))
    }

    pub fn flash(&self) -> &HashSet<(usize, usize)> {
        &self.flash
    }

    pub fn is_active(&self) -> bool {
        self.anim.is_some() || self.flash_until.is_some()
    }

    pub fn clear(&mut self) {
        self.anim = None;
        self.flash.clear();
        self.flash_until = None;
    }
}

fn changed_cells(old: &[Vec<Tile>], new: &[Vec<Tile>]) -> HashSet<(usize, usize)> {
    let mut set = HashSet::new();
    for (r, (old_row, new_row)) in old.iter().zip(new.iter()).enumerate() {
        for (c, (&ov, &nv)) in old_row.iter().zip(new_row.iter()).enumerate() {
            if nv != ov && nv != Tile::Empty {
                set.insert((r, c));
            }
        }
    }
    set
}

/// Returns the [`Color`] of the tile(used for rendering)
fn tile_color(tile: Tile) -> Color {
    match tile {
        Tile::Empty => Color::Rgb(180, 180, 180),
        Tile::Two => Color::Rgb(255, 220, 80),
        Tile::Four => Color::Rgb(255, 165, 30),
        Tile::Eight => Color::Rgb(255, 100, 20),
        Tile::Sixteen => Color::Rgb(240, 50, 50),
        Tile::ThirtyTwo => Color::Rgb(200, 20, 120),
        Tile::SixtyFour => Color::Rgb(150, 0, 200),
        Tile::OneHundredTwentyEight => Color::Rgb(80, 20, 220),
        Tile::TwoHundredFiftySix => Color::Rgb(20, 100, 255),
        Tile::FiveHundredTwelve => Color::Rgb(0, 200, 220),
        Tile::OneThousandTwentyFour => Color::Rgb(20, 220, 120),
        Tile::TwoThousandFourtyEight => Color::Rgb(255, 215, 0),
        Tile::FourHundredNinetySix => Color::Rgb(255, 255, 255),
    }
}

pub struct TermGuard(Terminal<CrosstermBackend<io::Stdout>>);

impl TermGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|e| {
            let _ = disable_raw_mode();
            e
        })?;
        let backend = CrosstermBackend::new(stdout);
        Terminal::new(backend)
            .map_err(|e| {
                let _ = disable_raw_mode();
                let _ = execute!(io::stdout(), LeaveAlternateScreen);
                io::Error::other(e.to_string())
            })
            .map(Self)
    }

    pub fn render_board(
        &mut self,
        game: &Game,
        message: Option<&str>,
        x_shift: i16,
        y_shift: i16,
        flash: &HashSet<(usize, usize)>,
    ) -> io::Result<()> {
        self.0
            .draw(|f| -> () {
                let board = game.board();
                let area = f.area();
                let board_size = board.len() as u16;
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

                for (row_i, row) in board.iter().enumerate() {
                    for (col_i, &val) in row.iter().enumerate() {
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
                        let is_flash = flash.contains(&(row_i, col_i)) && val != Tile::Empty;

                        let (fg, bg) = if val == Tile::Empty {
                            (EMPTY_BG, EMPTY_BG)
                        } else {
                            let base = tile_color(val);
                            let bg = if is_flash { brighten(base, 70) } else { base };
                            (Color::White, bg)
                        };

                        let block = Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .style(Style::default().bg(bg));

                        let inner = block.inner(cell_area);
                        f.render_widget(block, cell_area);

                        if val != Tile::Empty {
                            let text = Paragraph::new(Span::styled(
                                val.to_string(),
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

impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.0.backend_mut(), LeaveAlternateScreen);
        let _ = self.0.show_cursor();
    }
}
