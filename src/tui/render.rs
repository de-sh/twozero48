use rand::Rng;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use crate::{Board, Game, Status, Tile};

use super::MoveEffects;

const TILE_W: u16 = 14;
const TILE_H: u16 = 7;
const GUTTER_W: u16 = 2;
const GUTTER_H: u16 = 1;
const HEADER_H: u16 = 2;
const FOOTER_H: u16 = 2;
const OUTER_GUTTER_H: u16 = 1;
const BOARD_BG: Color = Color::Rgb(187, 173, 160);
const EMPTY_BG: Color = Color::Rgb(205, 193, 180);
const TILE_DARK_FG: Color = Color::Rgb(119, 110, 101);
const TILE_LIGHT_FG: Color = Color::Rgb(249, 246, 242);
const MUTED_FG: Color = Color::Rgb(142, 122, 102);
const TEXT_FG: Color = Color::Rgb(99, 89, 78);
const TITLE_TWO_FG: Color = Color::Rgb(238, 228, 218);
const TITLE_ZERO_FG: Color = Color::Rgb(237, 224, 200);
const TITLE_FOUR_FG: Color = Color::Rgb(242, 177, 121);
const TITLE_EIGHT_FG: Color = Color::Rgb(246, 124, 95);
const SCORE_VALUE_FG: Color = Color::Rgb(249, 246, 242);
const INVALID_MOVE_FG: Color = Color::Rgb(255, 170, 80);
const WIN_ACCENT: Color = Color::Rgb(255, 215, 90);
const WIN_BG: Color = Color::Rgb(36, 30, 10);
const LOSS_ACCENT: Color = Color::Rgb(255, 105, 130);
const LOSS_BG: Color = Color::Rgb(34, 16, 20);
const OVERLAY_TEXT_FG: Color = Color::Rgb(225, 225, 225);
const FLASH_BRIGHTEN: u8 = 70;
const TILE_BACKGROUNDS: [Color; 12] = [
    Color::Rgb(238, 228, 218),
    Color::Rgb(237, 224, 200),
    Color::Rgb(242, 177, 121),
    Color::Rgb(245, 149, 99),
    Color::Rgb(246, 124, 95),
    Color::Rgb(246, 94, 59),
    Color::Rgb(237, 207, 114),
    Color::Rgb(237, 204, 97),
    Color::Rgb(237, 200, 80),
    Color::Rgb(237, 197, 63),
    Color::Rgb(237, 194, 46),
    Color::Rgb(60, 58, 50),
];

#[derive(Clone, Copy)]
struct TilePalette {
    fg: Color,
    bg: Color,
}

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
    tile_palette(tile).bg
}

fn tile_palette(tile: Tile) -> TilePalette {
    let exponent = tile.exponent();
    let bg = match exponent {
        0 => EMPTY_BG,
        1..=12 => TILE_BACKGROUNDS[(exponent - 1) as usize],
        _ => Color::Rgb(45, 45, 40),
    };
    let fg = if exponent >= 3 {
        TILE_LIGHT_FG
    } else {
        TILE_DARK_FG
    };

    TilePalette { fg, bg }
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

/// Complete render input for one terminal frame.
pub struct Screen<'a, R: Rng> {
    game: &'a Game<R>,
    effects: &'a MoveEffects,
    valid_move: bool,
}

impl<'a, R: Rng> Screen<'a, R> {
    /// Creates a screen from stable game state plus transient move effects.
    pub fn new(game: &'a Game<R>, effects: &'a MoveEffects, valid_move: bool) -> Self {
        Self {
            game,
            effects,
            valid_move,
        }
    }

    /// Renders the frame into the provided Ratatui frame.
    pub fn render(&self, frame: &mut Frame<'_>) {
        let board = self.game.board();
        let layout = BoardLayout::new(frame.area(), board.size());

        Header::new(self.game.score(), self.game.largest_tile()).render(frame, layout.header);
        BoardGrid::new(board, self.effects, layout.grid_spec).render(
            frame,
            layout.terminal,
            layout.grid,
        );
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
    grid_spec: GridSpec,
}

/// Board geometry in terminal cells.
///
/// A terminal cell is roughly twice as tall as it is wide, so the horizontal
/// dimensions are doubled relative to the vertical ones. Inner vertical gutters
/// are drawn as shared one-row separators over the board background, so the
/// preferred 14x7 tile with a 2-column gutter approximates the original 2048
/// board's 7:1 tile-to-gap ratio while still scaling down for short terminals.
#[derive(Clone, Copy)]
struct GridSpec {
    tile_w: u16,
    tile_h: u16,
    gutter_w: u16,
    gutter_h: u16,
}

impl BoardLayout {
    fn new(terminal: Rect, board_size: usize) -> Self {
        let board_size = board_size as u16;
        let gutter_w = GUTTER_W;
        let gutter_h = GUTTER_H;
        let gutters_w = gutter_w * (board_size + 1);
        let gutters_h = gutter_h * board_size.saturating_sub(1) + OUTER_GUTTER_H * 2;
        let available_w = terminal.width.saturating_sub(gutters_w) / board_size;
        let available_h = terminal
            .height
            .saturating_sub(HEADER_H + FOOTER_H)
            .saturating_sub(gutters_h)
            / board_size;
        let tile_h = odd_tile_height(available_h.min(TILE_H));
        let tile_w = available_w.min(tile_h * 2).clamp(4, TILE_W);
        let grid_spec = GridSpec {
            tile_w,
            tile_h,
            gutter_w,
            gutter_h,
        };
        let grid_width = tile_w * board_size + gutters_w;
        let grid_height = tile_h * board_size + gutters_h;

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
            grid_spec,
        }
    }
}

fn odd_tile_height(height: u16) -> u16 {
    let height = height.max(1);
    // Odd tile bodies give the numeric label a real center row. Even-sized
    // growth is absorbed by the separator rows around the bodies.
    if height % 2 == 0 { height - 1 } else { height }
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
    grid_spec: GridSpec,
}

impl<'a> BoardGrid<'a> {
    fn new(board: &'a Board, effects: &'a MoveEffects, grid_spec: GridSpec) -> Self {
        Self {
            board,
            effects,
            grid_spec,
        }
    }

    fn render(&self, frame: &mut Frame<'_>, terminal_area: Rect, grid_rect: Rect) {
        frame.render_widget(Block::default().style(bg_style(BOARD_BG)), grid_rect);

        let board_size = self.board.size();
        for row in 0..board_size {
            for col in 0..board_size {
                let Some(area) = self.tile_rect(terminal_area, grid_rect, row, col) else {
                    continue;
                };

                let tile = if self.effects.hides_cell((row, col)) {
                    Tile::EMPTY
                } else {
                    self.board[(row, col)]
                };
                TileCell::new(
                    tile,
                    tile != Tile::EMPTY && self.effects.flash().contains(&(row, col)),
                )
                .render(frame, area);
            }
        }
        self.render_row_separators(frame, grid_rect);

        if self.effects.is_sliding() {
            for tile in self.effects.animated_tiles() {
                let Some(area) = self.animated_tile_rect(terminal_area, grid_rect, tile) else {
                    continue;
                };

                TileCell::new(tile.tile, false).render(frame, area);
            }
        }

        if let Some((row, col)) = self.effects.spawning_cell() {
            if let Some(area) = self
                .tile_rect(terminal_area, grid_rect, row, col)
                .and_then(|area| scaled_rect(area, self.effects.spawn_progress()))
            {
                TileCell::new(self.board[(row, col)], false).render(frame, area);
            }
        }
    }

    fn render_row_separators(&self, frame: &mut Frame<'_>, grid_rect: Rect) {
        let rows = self.board.size().saturating_sub(1) as u16;

        for row in 0..rows {
            let y = grid_rect.y
                + OUTER_GUTTER_H
                + self.grid_spec.tile_h
                + row * (self.grid_spec.tile_h + self.grid_spec.gutter_h);
            frame.render_widget(
                Block::default().style(bg_style(BOARD_BG)),
                Rect::new(grid_rect.x, y, grid_rect.width, 1),
            );
        }
    }

    fn tile_rect(
        &self,
        terminal_area: Rect,
        grid_rect: Rect,
        row: usize,
        col: usize,
    ) -> Option<Rect> {
        let x = grid_rect.x.checked_add_signed(
            self.grid_spec.gutter_w as i16
                + col as i16 * (self.grid_spec.tile_w + self.grid_spec.gutter_w) as i16,
        )?;
        let y = grid_rect.y.checked_add_signed(
            OUTER_GUTTER_H as i16
                + row as i16 * (self.grid_spec.tile_h + self.grid_spec.gutter_h) as i16,
        )?;
        let area = Rect::new(x, y, self.grid_spec.tile_w, self.grid_spec.tile_h);

        (area.intersection(terminal_area) == area).then_some(area)
    }

    fn animated_tile_rect(
        &self,
        terminal_area: Rect,
        grid_rect: Rect,
        tile: &super::AnimatedTile,
    ) -> Option<Rect> {
        let from = self.tile_rect(terminal_area, grid_rect, tile.from.0, tile.from.1)?;
        let to = self.tile_rect(terminal_area, grid_rect, tile.to.0, tile.to.1)?;
        let progress = self.effects.animation_progress();
        let x = interpolate(from.x, to.x, progress);
        let y = interpolate(from.y, to.y, progress);
        let area = Rect::new(x, y, self.grid_spec.tile_w, self.grid_spec.tile_h);

        (area.intersection(terminal_area) == area).then_some(area)
    }
}

fn interpolate(from: u16, to: u16, progress: f32) -> u16 {
    let from = f32::from(from);
    let to = f32::from(to);

    (from + (to - from) * progress)
        .round()
        .clamp(0.0, f32::from(u16::MAX)) as u16
}

/// Scales a spawned tile out from the center of its final cell.
fn scaled_rect(area: Rect, progress: f32) -> Option<Rect> {
    if progress <= 0.0 {
        return None;
    }

    let width = scaled_dimension(area.width, progress);
    let height = scaled_dimension(area.height, progress);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;

    Some(Rect::new(x, y, width, height))
}

fn scaled_dimension(size: u16, progress: f32) -> u16 {
    ((f32::from(size) * progress).ceil() as u16).clamp(1, size)
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
        let palette = self.palette();

        frame.render_widget(Block::default().style(bg_style(palette.bg)), area);
        if self.tile == Tile::EMPTY {
            return;
        }

        let text = self.tile.to_string();
        let text_width = text.chars().count() as u16;
        let text_x = area.x + area.width.saturating_sub(text_width) / 2;
        let text_y = area.y + area.height / 2;

        frame.render_widget(
            Paragraph::new(Span::styled(
                text,
                fg_bg(palette.fg, palette.bg).add_modifier(Modifier::BOLD),
            )),
            Rect::new(text_x, text_y, text_width, 1),
        );
    }

    fn palette(&self) -> TilePalette {
        let mut palette = tile_palette(self.tile);
        if self.flashing && self.tile != Tile::EMPTY {
            palette.bg = brighten(palette.bg, FLASH_BRIGHTEN);
        }
        palette
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
        frame.render_widget(
            Paragraph::new(self.line())
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            area,
        );
    }
}

fn render_centered_line(frame: &mut Frame<'_>, line: Line<'_>, area: Rect) {
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

/// Game-end overlay content.
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

    /// Creates the win overlay for a completed game.
    pub fn won(score: usize, winning: Tile) -> Self {
        Self {
            title: "You won! Great job.",
            detail: format!("Score {score} · Reached {winning}"),
            accent: WIN_ACCENT,
            background: WIN_BG,
        }
    }

    /// Creates the loss overlay for a completed game.
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
    fn tile_numbers_follow_original_foreground_split() {
        assert_eq!(tile_palette(Tile::TWO).fg, TILE_DARK_FG);
        assert_eq!(tile_palette(Tile::FOUR).fg, TILE_DARK_FG);
        assert_eq!(tile_palette(Tile::new(3)).fg, TILE_LIGHT_FG);
        assert_eq!(tile_palette(Tile::new(11)).fg, TILE_LIGHT_FG);
        assert_eq!(tile_palette(Tile::new(12)).fg, TILE_LIGHT_FG);
    }

    #[test]
    fn tile_numbers_center_left_when_width_is_uneven() {
        let mut tui = test_tui(10, 5);

        tui.0
            .draw(|frame| TileCell::new(Tile::new(7), false).render(frame, frame.area()))
            .expect("failed to draw tile");

        let buffer = tui.0.backend().buffer();
        assert_eq!(buffer[(3, 2)].symbol(), "1");
        assert_eq!(buffer[(4, 2)].symbol(), "2");
        assert_eq!(buffer[(5, 2)].symbol(), "8");
        assert_eq!(buffer[(6, 2)].symbol(), " ");
    }

    #[test]
    fn grid_gutters_match_outer_margins() {
        let board = Board::new(4).expect("board created");
        let effects = MoveEffects::default();
        let grid = BoardGrid::new(
            &board,
            &effects,
            GridSpec {
                tile_w: TILE_W,
                tile_h: TILE_H,
                gutter_w: GUTTER_W,
                gutter_h: GUTTER_H,
            },
        );
        let terminal = Rect::new(0, 0, 80, 40);
        let grid_rect = Rect::new(10, 20, 66, 30);

        assert_eq!(
            grid.tile_rect(terminal, grid_rect, 0, 0),
            Some(Rect::new(12, 21, 14, 7))
        );
        assert_eq!(
            grid.tile_rect(terminal, grid_rect, 0, 1),
            Some(Rect::new(28, 21, 14, 7))
        );
        assert_eq!(
            grid.tile_rect(terminal, grid_rect, 0, 3),
            Some(Rect::new(60, 21, 14, 7))
        );
    }

    #[test]
    fn layout_prefers_seven_to_one_tile_gutters_when_roomy() {
        let layout = BoardLayout::new(Rect::new(0, 0, 80, 40), 4);

        assert_eq!(layout.grid_spec.tile_w, 14);
        assert_eq!(layout.grid_spec.tile_h, 7);
        assert_eq!(layout.grid_spec.gutter_w, 2);
        assert_eq!(layout.grid_spec.gutter_h, 1);
        assert_eq!(layout.grid.height, 33);
    }

    #[test]
    fn layout_scales_down_to_fit_short_terminals() {
        let layout = BoardLayout::new(Rect::new(0, 0, 80, 23), 4);

        assert_eq!(layout.grid_spec.tile_w, 6);
        assert_eq!(layout.grid_spec.tile_h, 3);
        assert_eq!(layout.grid.width, 34);
        assert_eq!(layout.grid.height, 17);
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
        let first_grid_line = buffer_line(buffer, 3);
        let grid_left = 23;

        assert!(score_line.contains("SCORE"), "{score_line}");
        assert!(!score_line.contains("╭"), "{score_line}");
        assert!(!first_grid_line.contains("╭"), "{first_grid_line}");
        assert_eq!(buffer[(grid_left, 3)].bg, BOARD_BG);
        assert_eq!(buffer[(grid_left + 1, 3)].bg, BOARD_BG);
        assert_eq!(buffer[(grid_left + 2, 3)].bg, BOARD_BG);
        assert_eq!(buffer[(grid_left + 2, 4)].bg, EMPTY_BG);
        assert_eq!(buffer[(grid_left + 2, 5)].bg, EMPTY_BG);
    }

    #[test]
    fn visual_grid_rows_stay_stable_at_short_size() {
        assert_visual_grid(
            80,
            23,
            34,
            17,
            6,
            3,
            [
                "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                "BB......BB......BB......BB......BB",
                "BB......BB......BB......BB......BB",
                "BB......BB......BB......BB......BB",
                "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                "BB......BB......BB......BB......BB",
                "BB......BB......BB......BB......BB",
                "BB......BB......BB......BB......BB",
                "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                "BB......BB......BB......BB......BB",
                "BB......BB......BB......BB......BB",
                "BB......BB......BB......BB......BB",
                "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                "BB......BB......BB......BB......BB",
                "BB......BB......BB......BB......BB",
                "BB......BB......BB......BB......BB",
                "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
            ],
        );
    }

    #[test]
    fn visual_grid_rows_stay_stable_at_roomy_size() {
        assert_visual_grid(
            80,
            40,
            66,
            33,
            14,
            7,
            [
                "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BB..............BB..............BB..............BB..............BB",
                "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
            ],
        );
    }

    fn assert_visual_grid<const H: usize>(
        width: u16,
        height: u16,
        grid_w: u16,
        grid_h: u16,
        tile_w: u16,
        tile_h: u16,
        expected: [&str; H],
    ) {
        let board = Board::new(4).expect("board created");
        let mut tui = test_tui(width, height);
        let game = Game::from_parts(board, Tile::new(11), 0);
        let effects = MoveEffects::default();

        let screen = Screen::new(&game, &effects, true);
        tui.draw(screen).expect("failed to draw screen");

        let buffer = tui.0.backend().buffer();
        let layout = BoardLayout::new(Rect::new(0, 0, width, height), 4);
        assert_eq!(layout.grid.width, grid_w);
        assert_eq!(layout.grid.height, grid_h);
        assert_eq!(layout.grid_spec.tile_w, tile_w);
        assert_eq!(layout.grid_spec.tile_h, tile_h);
        assert_eq!(grid_visual(buffer, layout.grid), expected.join("\n"));
    }

    fn grid_visual(buffer: &Buffer, grid: Rect) -> String {
        let mut view = String::new();
        for y in grid.y..grid.y + grid.height {
            if y > grid.y {
                view.push('\n');
            }
            for x in grid.x..grid.x + grid.width {
                let cell = &buffer[(x, y)];
                if cell.bg == BOARD_BG {
                    view.push('B');
                } else if cell.bg == EMPTY_BG {
                    view.push('.');
                } else {
                    view.push(' ');
                }
            }
        }
        view
    }
}
