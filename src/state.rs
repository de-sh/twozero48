use std::{
    collections::HashSet,
    io,
    time::{Duration, Instant},
};

use ratatui::backend::{Backend, CrosstermBackend};

use crate::{Board, Game, Move, Play, Status, Tile, Tui, TuiWriter};

const FLASH_DURATION: Duration = Duration::from_millis(120);

pub struct State<P = Game, B: Backend = CrosstermBackend<TuiWriter>> {
    terminal: Tui<B>,
    game: P,
    move_effects: MoveEffects,
    valid_move: bool,
    previous_largest: Tile,
}

impl State<Game, CrosstermBackend<TuiWriter>> {
    pub fn new(game: Game) -> io::Result<Self> {
        Ok(Self::with_tui(game, Tui::new()?))
    }
}

impl<P: Play, B: Backend> State<P, B> {
    pub fn with_tui(game: P, terminal: Tui<B>) -> Self {
        Self {
            terminal,
            game,
            move_effects: MoveEffects::new(),
            valid_move: true,
            previous_largest: Tile::EMPTY,
        }
    }

    pub fn tick_effects(&mut self) {
        self.move_effects.tick();
    }

    pub fn effects_active(&self) -> bool {
        self.move_effects.is_active()
    }

    pub fn clear_invalid_move(&mut self) {
        self.valid_move = true;
    }

    pub fn apply_move(&mut self, mov: Move) -> Status {
        if matches!(mov, Move::Dont) {
            self.valid_move = false;
            return self.game.status();
        }

        let old_board = self.game.board().clone();

        self.valid_move = self.game.mover(mov);

        if self.valid_move {
            self.move_effects
                .record_move(mov, &old_board, self.game.board());
            self.game.spawn();

            let current_largest = self.game.largest_tile();
            self.check_milestone(current_largest);
        }

        self.game.status()
    }

    pub fn render_board(&mut self) -> io::Result<()> {
        let (x_shift, y_shift) = self.move_effects.shift();
        let message = if self.valid_move {
            None
        } else {
            Some("No tiles moved — try a different direction")
        };

        self.terminal.render_board(
            &self.game,
            message,
            x_shift,
            y_shift,
            self.move_effects.flash(),
        )
    }

    pub fn render_end_message(&mut self, message: &str) -> io::Result<()> {
        self.move_effects.clear();
        self.terminal
            .render_board(&self.game, Some(message), 0, 0, self.move_effects.flash())
    }

    fn check_milestone(&mut self, tile: Tile) {
        if tile.exponent() <= self.previous_largest.exponent() {
            return;
        }
        self.previous_largest = tile;
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

#[derive(Default)]
struct MoveEffects {
    anim: Option<AnimState>,
    flash: HashSet<(usize, usize)>,
    flash_until: Option<Instant>,
}

impl MoveEffects {
    fn new() -> Self {
        Self::default()
    }

    fn record_move(&mut self, mov: Move, old_board: &Board, new_board: &Board) {
        self.anim = AnimState::new(mov);
        self.flash = changed_cells(old_board, new_board);
        self.flash_until = Some(Instant::now() + FLASH_DURATION);
    }

    fn tick(&mut self) {
        let now = Instant::now();
        if self.flash_until.is_some_and(|until| now >= until) {
            self.flash.clear();
            self.flash_until = None;
        }

        if self.anim.as_ref().is_some_and(AnimState::expired) {
            self.anim = None;
        }
    }

    fn shift(&self) -> (i16, i16) {
        self.anim.as_ref().map_or((0, 0), AnimState::shift)
    }

    fn flash(&self) -> &HashSet<(usize, usize)> {
        &self.flash
    }

    fn is_active(&self) -> bool {
        self.anim.is_some() || self.flash_until.is_some()
    }

    fn clear(&mut self) {
        self.anim = None;
        self.flash.clear();
        self.flash_until = None;
    }
}

fn changed_cells(old: &Board, new: &Board) -> HashSet<(usize, usize)> {
    let mut set = HashSet::new();
    let size = old.size().min(new.size());

    for row in 0..size {
        for col in 0..size {
            let old_tile = old[(row, col)];
            let new_tile = new[(row, col)];
            if new_tile != old_tile && new_tile != Tile::EMPTY {
                set.insert((row, col));
            }
        }
    }

    set
}
