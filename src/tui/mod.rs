use std::{collections::HashSet, time::Duration};

use rand::{Rng, rngs::ThreadRng};
use web_time::Instant;

pub use render::Screen;
pub use terminal::Tui;

use crate::{Board, Game, Input, Move, Status, Tile};

#[cfg(not(target_arch = "wasm32"))]
mod native;
pub(crate) mod render;
mod terminal;

const FLASH_DURATION: Duration = Duration::from_millis(120);

pub struct State<R: Rng = ThreadRng> {
    game: Game<R>,
    move_effects: MoveEffects,
    valid_move: bool,
}

impl<R: Rng> State<R> {
    pub fn new(game: Game<R>) -> Self {
        Self {
            game,
            move_effects: MoveEffects::default(),
            valid_move: true,
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

    pub fn clear_effects(&mut self) {
        self.move_effects.clear();
    }

    pub fn apply_move(&mut self, mov: Move) -> Status {
        let old_board = self.game.board().clone();

        self.valid_move = self.game.mover(mov);

        if self.valid_move {
            self.move_effects
                .record_move(mov, &old_board, self.game.board());
            self.game.spawn();
        }

        self.game.status()
    }

    pub fn as_screen(&self) -> Screen<'_, R> {
        Screen::new(&self.game, &self.move_effects, self.valid_move)
    }

    pub fn status(&self) -> Status {
        self.game.status()
    }

    pub fn handle_input(&mut self, input: Input) {
        match input {
            Input::Quit => unreachable!("Ensure never passed to handle_input!!"),
            Input::Restart => {
                self.reset();
                self.clear_effects();
                self.clear_invalid_move();
            }
            Input::Ignore => {
                self.clear_invalid_move();
            }
            Input::Move(mov) if self.status() == Status::On => {
                if self.apply_move(mov) != Status::On {
                    self.clear_effects();
                }
            }
            Input::Move(_) => {
                self.clear_invalid_move();
            }
        }
    }

    pub fn reset(&mut self) {
        self.game.reset();
    }
}

struct AnimState {
    dx: i16,
    dy: i16,
    started: Instant,
}

impl AnimState {
    fn new(mov: Move) -> Self {
        let (dx, dy) = match mov {
            Move::Left => (-2, 0),
            Move::Right => (2, 0),
            Move::Up => (0, -2),
            Move::Down => (0, 2),
        };

        Self {
            dx,
            dy,
            started: Instant::now(),
        }
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
pub struct MoveEffects {
    anim: Option<AnimState>,
    flash: HashSet<(usize, usize)>,
    flash_until: Option<Instant>,
}

impl MoveEffects {
    pub(crate) fn record_move(&mut self, mov: Move, old_board: &Board, new_board: &Board) {
        self.anim = Some(AnimState::new(mov));
        self.flash = changed_cells(old_board, new_board);
        self.flash_until = Some(Instant::now() + FLASH_DURATION);
    }

    pub(crate) fn tick(&mut self) {
        let now = Instant::now();
        if self.flash_until.is_some_and(|until| now >= until) {
            self.flash.clear();
            self.flash_until = None;
        }

        if self.anim.as_ref().is_some_and(AnimState::expired) {
            self.anim = None;
        }
    }

    pub(crate) fn shift(&self) -> (i16, i16) {
        self.anim.as_ref().map_or((0, 0), AnimState::shift)
    }

    pub(crate) fn flash(&self) -> &HashSet<(usize, usize)> {
        &self.flash
    }

    pub(crate) fn is_active(&self) -> bool {
        self.anim.is_some() || self.flash_until.is_some()
    }

    pub(crate) fn clear(&mut self) {
        self.anim = None;
        self.flash.clear();
        self.flash_until = None;
    }
}

fn changed_cells(old: &Board, new: &Board) -> HashSet<(usize, usize)> {
    let size = old.size().min(new.size());
    (0..size)
        .flat_map(|r| (0..size).map(move |c| (r, c)))
        .filter(|&(r, c)| {
            let t = new[(r, c)];
            t != old[(r, c)] && t != Tile::EMPTY
        })
        .collect()
}
