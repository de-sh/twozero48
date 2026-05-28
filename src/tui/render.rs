use rand::Rng;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::{Board, Game, Status, Tile};

use super::MoveEffects;

const CELL_W: u16 = 10;
const CELL_H: u16 = 5;
const HEADER_H: u16 = 2;
const FOOTER_H: u16 = 1;
const EMPTY_BG: Color = Color::Rgb(40, 40, 40);
const MUTED_FG: Color = Color::Rgb(100, 100, 100);
const TEXT_FG: Color = Color::Rgb(200, 200, 200);
const TITLE_TWO_FG: Color = Color::Rgb(255, 100, 20);
const TITLE_ZERO_FG: Color = Color::Rgb(255, 150, 20);
const TITLE_FOUR_FG: Color = Color::Rgb(200, 20, 120);
const TITLE_EIGHT_FG: Color = Color::Rgb(80, 20, 220);
const SCORE_VALUE_FG: Color = Color::Rgb(230, 230, 230);
const INVALID_MOVE_FG: Color = Color::Rgb(255, 170, 80);
const WIN_ACCENT: Color = Color::Rgb(255, 215, 90);
const WIN_BG: Color = Color::Rgb(36, 30, 10);
const LOSS_ACCENT: Color = Color::Rgb(255, 105, 130);
const LOSS_BG: Color = Color::Rgb(34, 16, 20);
const OVERLAY_TEXT_FG: Color = Color::Rgb(225, 225, 225);
const FLASH_BRIGHTEN: u8 = 70;

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

/// Color used to render a tile.
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

fn fg(color: Color) -> Style {
    Style::default().fg(color)
}

fn bg_style(color: Color) -> Style {
    Style::default().bg(color)
}

fn fg_bg(fg: Color, bg: Color) -> Style {
    Style::default().fg(fg).bg(bg)
}

fn bold_fg(color: Color) -> Style {
    fg(color).add_modifier(Modifier::BOLD)
}

pub struct Screen<'a, R: Rng> {
    game: &'a Game<R>,
    effects: &'a MoveEffects,
    valid_move: bool,
}

impl<'a, R: Rng> Screen<'a, R> {
    pub fn new(game: &'a Game<R>, effects: &'a MoveEffects, valid_move: bool) -> Self {
        Self {
            game,
            effects,
            valid_move,
        }
    }

    pub fn render(&self, frame: &mut Frame<'_>) {
        let board = self.game.board();
        let layout = BoardLayout::new(frame.area(), board.size());

        Header::new(self.game.score(), self.game.largest_tile()).render(frame, layout.header);
        BoardGrid::new(board, self.effects).render(frame, layout.terminal, layout.grid);
        if let Ok(overlay) = Overlay::try_from(self.game) {
            overlay.render(frame, layout.playfield, layout.grid);
        }
        Footer::try_from(self.game, self.valid_move).render(frame, layout.footer);
    }
}

struct BoardLayout {
    terminal: Rect,
    header: Rect,
    playfield: Rect,
    footer: Rect,
    grid: Rect,
}

impl BoardLayout {
    fn new(terminal: Rect, board_size: usize) -> Self {
        let board_size = board_size as u16;
        let grid_width = CELL_W * board_size;
        let grid_height = CELL_H * board_size;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(HEADER_H),
                Constraint::Min(grid_height),
                Constraint::Length(FOOTER_H),
            ])
            .split(terminal);

        let playfield = chunks[1];
        let grid = Rect::new(
            playfield.x + playfield.width.saturating_sub(grid_width) / 2,
            playfield.y + playfield.height.saturating_sub(grid_height) / 2,
            grid_width,
            grid_height,
        );

        Self {
            terminal,
            header: chunks[0],
            playfield,
            footer: chunks[2],
            grid,
        }
    }
}

struct Header {
    score: usize,
    largest: Tile,
}

impl Header {
    fn new(score: usize, largest: Tile) -> Self {
        Self { score, largest }
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        render_centered_line(
            frame,
            Line::from(vec![
                Span::styled("2", bold_fg(TITLE_TWO_FG)),
                Span::raw("  "),
                Span::styled("0", bold_fg(TITLE_ZERO_FG)),
                Span::raw("  "),
                Span::styled("4", bold_fg(TITLE_FOUR_FG)),
                Span::raw("  "),
                Span::styled("8", bold_fg(TITLE_EIGHT_FG)),
            ]),
            Self::line_area(area, 0),
        );
        render_centered_line(frame, self.score_line(), Self::line_area(area, 1));
    }

    fn score_line(&self) -> Line<'static> {
        let mut score = vec![
            Span::styled("SCORE  ", fg(MUTED_FG)),
            Span::styled(self.score.to_string(), bold_fg(SCORE_VALUE_FG)),
        ];
        if self.largest >= Tile::ONE_TWO_EIGHT {
            score.extend([
                Span::styled("   ·   MILESTONE  ", fg(MUTED_FG)),
                Span::styled(self.largest.to_string(), bold_fg(tile_color(self.largest))),
            ]);
        }

        Line::from(score)
    }

    fn line_area(area: Rect, row: u16) -> Rect {
        Rect::new(area.x, area.y + row, area.width, 1)
    }
}

struct BoardGrid<'a> {
    board: &'a Board,
    effects: &'a MoveEffects,
}

impl<'a> BoardGrid<'a> {
    fn new(board: &'a Board, effects: &'a MoveEffects) -> Self {
        Self { board, effects }
    }

    fn render(&self, frame: &mut Frame<'_>, terminal_area: Rect, grid_rect: Rect) {
        let board_size = self.board.size();
        for row in 0..board_size {
            for col in 0..board_size {
                let Some(area) = self.shifted_cell_rect(terminal_area, grid_rect, row, col) else {
                    continue;
                };

                let tile = self.board[(row, col)];
                TileCell::new(
                    tile,
                    tile != Tile::EMPTY && self.effects.flash().contains(&(row, col)),
                )
                .render(frame, area);
            }
        }
    }

    fn shifted_cell_rect(
        &self,
        terminal_area: Rect,
        grid_rect: Rect,
        row: usize,
        col: usize,
    ) -> Option<Rect> {
        let (x_shift, y_shift) = self.effects.shift();
        let x = grid_rect
            .x
            .checked_add_signed(col as i16 * CELL_W as i16 + x_shift)?;
        let y = grid_rect
            .y
            .checked_add_signed(row as i16 * CELL_H as i16 + y_shift)?;
        let area = Rect::new(x, y, CELL_W, CELL_H);

        (area.intersection(terminal_area) == area).then_some(area)
    }
}

struct TileCell {
    tile: Tile,
    flashing: bool,
}

impl TileCell {
    fn new(tile: Tile, flashing: bool) -> Self {
        Self { tile, flashing }
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        let bg = self.background();
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(bg_style(bg));
        let inner = block.inner(area);

        frame.render_widget(block, area);
        if self.tile == Tile::EMPTY {
            return;
        }

        frame.render_widget(
            Paragraph::new(Span::styled(
                self.tile.to_string(),
                fg_bg(Color::White, bg).add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
            Rect::new(inner.x, inner.y + inner.height / 2, inner.width, 1),
        );
    }

    fn background(&self) -> Color {
        if self.tile == Tile::EMPTY {
            return EMPTY_BG;
        }

        let color = tile_color(self.tile);
        if self.flashing {
            brighten(color, FLASH_BRIGHTEN)
        } else {
            color
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Footer {
    ExitPrompt,
    InvalidMove,
    Controls { winning: Tile },
}

impl Footer {
    fn try_from<R: Rng>(game: &Game<R>, valid_move: bool) -> Self {
        match (game.status(), valid_move) {
            (Status::On, false) => Self::InvalidMove,
            (Status::On, true) => Self::Controls {
                winning: game.winning(),
            },
            _ => Self::ExitPrompt,
        }
    }

    fn line(&self) -> Line<'static> {
        match self {
            Self::ExitPrompt => Line::from(vec![
                Span::styled("R", bold_fg(TEXT_FG)),
                Span::raw(": restart  ·  "),
                Span::styled("Q", bold_fg(TEXT_FG)),
                Span::raw(": quit"),
            ]),
            Self::InvalidMove => Line::from(vec![
                Span::styled("No tiles moved", bold_fg(INVALID_MOVE_FG)),
                Span::raw(" · try another direction"),
            ]),
            Self::Controls { winning } => Line::from(vec![
                Span::styled("WASD", bold_fg(TEXT_FG)),
                Span::raw("/ arrows / swipe: move"),
                Span::raw("  ·  "),
                Span::styled("R", bold_fg(TEXT_FG)),
                Span::raw(": restart  ·  "),
                Span::styled("Q", bold_fg(TEXT_FG)),
                Span::raw(": quit  ·  Win: "),
                Span::styled(winning.to_string(), bold_fg(tile_color(*winning))),
            ]),
        }
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        render_centered_line(frame, self.line(), area);
    }
}

fn render_centered_line(frame: &mut Frame<'_>, line: Line<'_>, area: Rect) {
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

#[derive(Clone, Debug)]
pub struct Overlay {
    title: &'static str,
    detail: String,
    accent: Color,
    background: Color,
}

impl Overlay {
    fn try_from<R: Rng>(game: &Game<R>) -> Result<Self, ()> {
        let overlay = match game.status() {
            Status::Won => Self::won(game.score(), game.largest_tile()),
            Status::Lost => Self::lost(game.score(), game.largest_tile()),
            _ => return Err(()),
        };

        Ok(overlay)
    }

    pub fn won(score: usize, winning: Tile) -> Self {
        Self {
            title: "You won! Great job.",
            detail: format!("Score {score} · Reached {winning}"),
            accent: WIN_ACCENT,
            background: WIN_BG,
        }
    }

    pub fn lost(score: usize, largest: Tile) -> Self {
        Self {
            title: "Game over :(",
            detail: format!("Score {score} · best tile {largest}"),
            accent: LOSS_ACCENT,
            background: LOSS_BG,
        }
    }

    fn render(&self, frame: &mut Frame<'_>, playfield_area: Rect, grid_rect: Rect) {
        if playfield_area.width == 0 || playfield_area.height == 0 {
            return;
        }

        let area = self.rect(playfield_area, grid_rect);
        let style = fg_bg(self.accent, self.background);
        let bg = bg_style(self.background);
        let title_style = style.add_modifier(Modifier::BOLD);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(style)
            .style(bg);

        frame.render_widget(Clear, area);

        let lines = vec![
            Line::from(Span::styled(self.title, title_style)),
            Line::from(Span::styled(
                &self.detail,
                fg_bg(OVERLAY_TEXT_FG, self.background),
            )),
        ];

        frame.render_widget(
            Paragraph::new(lines)
                .style(bg)
                .block(block)
                .alignment(Alignment::Center),
            area,
        );
    }

    fn rect(&self, playfield_area: Rect, grid_rect: Rect) -> Rect {
        let content_width = self.title.chars().count().max(self.detail.chars().count()) as u16;
        let width = content_width.saturating_add(6).min(playfield_area.width);
        let height = 4.min(playfield_area.height);
        let center_x = grid_rect.x + grid_rect.width / 2;
        let center_y = grid_rect.y + grid_rect.height / 2;
        let max_x = playfield_area.x + playfield_area.width.saturating_sub(width);
        let max_y = playfield_area.y + playfield_area.height.saturating_sub(height);

        Rect::new(
            center_x
                .saturating_sub(width / 2)
                .clamp(playfield_area.x, max_x),
            center_y
                .saturating_sub(height / 2)
                .clamp(playfield_area.y, max_y),
            width,
            height,
        )
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{backend::TestBackend, buffer::Buffer};

    use super::*;
    use crate::{Game, tui::Tui};

    fn test_tui(width: u16, height: u16) -> Tui<TestBackend> {
        Tui::from_backend(TestBackend::new(width, height))
            .expect("constructing TestBackend terminal failed")
    }

    fn buffer_line(buffer: &Buffer, y: u16) -> String {
        let mut line = String::new();
        for x in 0..buffer.area.width {
            line.push_str(buffer[(x, y)].symbol());
        }
        line
    }

    fn buffer_text(buffer: &Buffer) -> String {
        let mut text = String::new();
        for y in 0..buffer.area.height {
            text.push_str(&buffer_line(buffer, y));
            text.push('\n');
        }
        text
    }

    #[test]
    fn footer_content_tracks_render_state() {
        let winning = Tile::new(11);
        let board = Board::new(4).expect("board created");

        let game = Game::from_parts(board, winning, 0);
        assert_eq!(Footer::try_from(&game, true), Footer::Controls { winning });
        assert_eq!(Footer::try_from(&game, false), Footer::InvalidMove);

        let mut won_board = Board::new(4).expect("board created");
        won_board[(0, 0)] = winning;
        let game = Game::from_parts(won_board, winning, 0);
        assert_eq!(Footer::try_from(&game, true), Footer::ExitPrompt);
    }

    #[test]
    fn loss_overlay_reports_largest_tile_not_winning_tile() {
        let board = Board::from_grid([
            [Tile::new(10), Tile::TWO, Tile::new(10), Tile::TWO],
            [Tile::FOUR, Tile::new(3), Tile::FOUR, Tile::new(3)],
            [Tile::new(10), Tile::TWO, Tile::new(10), Tile::TWO],
            [Tile::FOUR, Tile::new(3), Tile::FOUR, Tile::new(3)],
        ])
        .expect("board ready");
        let game = Game::from_parts(board, Tile::new(11), 1234);
        let mut tui = test_tui(80, 24);
        let effects = MoveEffects::default();

        let screen = Screen::new(&game, &effects, true);
        tui.draw(screen).expect("failed to draw screen");

        let text = buffer_text(tui.0.backend().buffer());
        assert!(text.contains("Game over :("), "{text}");
        assert!(text.contains("Score 1234 · best tile 1024"), "{text}");
        assert!(!text.contains("best tile 2048"), "{text}");
    }

    #[test]
    fn render_keeps_grid_below_score_header_at_minimum_board_height() {
        let board = Board::new(4).expect("board created");
        let mut tui = test_tui(80, 23);
        let game = Game::from_parts(board, Tile::new(11), 1234);
        let effects = MoveEffects::default();

        let screen = Screen::new(&game, &effects, true);
        tui.draw(screen).expect("failed to draw screen");

        let buffer = tui.0.backend().buffer();
        let score_line = buffer_line(buffer, 1);
        let first_grid_line = buffer_line(buffer, 2);

        assert!(score_line.contains("SCORE"), "{score_line}");
        assert!(!score_line.contains("╭"), "{score_line}");
        assert!(first_grid_line.contains("╭"), "{first_grid_line}");
    }
}
