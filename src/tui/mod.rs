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
const SLIDE_DURATION: Duration = Duration::from_millis(140);
const SPAWN_DURATION: Duration = Duration::from_millis(90);

/// UI-facing game state plus transient render effects.
///
/// This wraps the pure game model with input handling and animation state so
/// frontends can tick, render, and reset from one stable object.
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
            let board_before_spawn = self.game.board().clone();
            self.game.spawn();
            self.move_effects
                .record_spawn(&board_before_spawn, self.game.board());
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AnimatedTile {
    /// The visible tile value before any merge promotion at the destination.
    pub(crate) tile: Tile,
    /// Source cell in the board before applying the move.
    pub(crate) from: (usize, usize),
    /// Destination cell after applying compression and merges.
    pub(crate) to: (usize, usize),
}

struct AnimState {
    tiles: Vec<AnimatedTile>,
    started: Instant,
}

impl AnimState {
    fn new(mov: Move, old_board: &Board) -> Self {
        Self {
            tiles: animated_tiles(mov, old_board),
            started: Instant::now(),
        }
    }

    fn progress(&self) -> f32 {
        (self.started.elapsed().as_secs_f32() / SLIDE_DURATION.as_secs_f32()).clamp(0.0, 1.0)
    }

    fn spawn_progress(&self) -> f32 {
        let elapsed = self.started.elapsed().saturating_sub(SLIDE_DURATION);
        (elapsed.as_secs_f32() / SPAWN_DURATION.as_secs_f32()).clamp(0.0, 1.0)
    }

    fn expired(&self) -> bool {
        self.started.elapsed() >= SLIDE_DURATION + SPAWN_DURATION
    }
}

/// Transient visual effects derived from the most recent move.
///
/// Effects are intentionally separate from [`Game`] so animation never changes
/// the game rules or board state.
#[derive(Default)]
pub struct MoveEffects {
    anim: Option<AnimState>,
    flash: HashSet<(usize, usize)>,
    flash_until: Option<Instant>,
    spawn: Option<(usize, usize)>,
}

impl MoveEffects {
    /// Records the slide and merge effects caused by applying a directional move.
    ///
    /// The slide animation is derived from the old board because the new board no
    /// longer contains enough information to distinguish a tile that slid into a
    /// cell from a tile that was already there.
    pub(crate) fn record_move(&mut self, mov: Move, old_board: &Board, new_board: &Board) {
        self.anim = Some(AnimState::new(mov, old_board));
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
            self.spawn = None;
        }
    }

    pub(crate) fn record_spawn(&mut self, old_board: &Board, new_board: &Board) {
        self.spawn = spawned_cells(old_board, new_board);
    }

    pub(crate) fn animated_tiles(&self) -> &[AnimatedTile] {
        self.anim.as_ref().map_or(&[], |anim| anim.tiles.as_slice())
    }

    pub(crate) fn animation_progress(&self) -> f32 {
        self.anim.as_ref().map_or(1.0, AnimState::progress)
    }

    pub(crate) fn spawn_progress(&self) -> f32 {
        self.anim.as_ref().map_or(1.0, AnimState::spawn_progress)
    }

    pub(crate) fn is_sliding(&self) -> bool {
        self.animation_progress() < 1.0
    }

    pub(crate) fn spawning_cell(&self) -> Option<(usize, usize)> {
        self.spawn
    }

    pub(crate) fn hides_cell(&self, cell: (usize, usize)) -> bool {
        (self.is_sliding() && self.animated_tiles().iter().any(|tile| tile.to == cell))
            || (self.spawn_progress() < 1.0 && self.spawn == Some(cell))
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
        self.spawn = None;
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

fn spawned_cells(old: &Board, new: &Board) -> Option<(usize, usize)> {
    let size = old.size().min(new.size());
    (0..size)
        .flat_map(|r| (0..size).map(move |c| (r, c)))
        .find(|&(r, c)| old[(r, c)] == Tile::EMPTY && new[(r, c)] != Tile::EMPTY)
}

fn animated_tiles(mov: Move, old: &Board) -> Vec<AnimatedTile> {
    let size = old.size();
    (0..size)
        .flat_map(|lane| animated_lane_tiles(mov, old, lane))
        .collect()
}

/// Replays one lane of 2048 compression to recover per-tile slide paths.
///
/// `Board::compress_lane` intentionally mutates in place and returns only score,
/// so animation keeps this read-only mirror that records where each old tile
/// travels. Both tiles in a merge animate to the same destination; the promoted
/// tile is then shown by the settled board after the slide phase completes.
fn animated_lane_tiles(mov: Move, board: &Board, lane: usize) -> Vec<AnimatedTile> {
    let positions = lane_positions(mov, board.size(), lane);
    let tiles: Vec<_> = positions
        .iter()
        .copied()
        .filter_map(|pos| {
            let tile = board[pos];
            (tile != Tile::EMPTY).then_some((pos, tile))
        })
        .collect();

    let mut animated = Vec::new();
    let mut write = 0;
    let mut read = 0;
    while read < tiles.len() {
        let (from, tile) = tiles[read];
        let to = positions[write];

        if tiles.get(read + 1).is_some_and(|(_, next)| *next == tile) {
            animated.push(AnimatedTile { tile, from, to });
            animated.push(AnimatedTile {
                tile,
                from: tiles[read + 1].0,
                to,
            });
            read += 2;
        } else {
            if from != to {
                animated.push(AnimatedTile { tile, from, to });
            }
            read += 1;
        }

        write += 1;
    }

    animated
}

fn lane_positions(mov: Move, size: usize, lane: usize) -> Vec<(usize, usize)> {
    match mov {
        Move::Left => (0..size).map(|col| (lane, col)).collect(),
        Move::Right => (0..size).rev().map(|col| (lane, col)).collect(),
        Move::Up => (0..size).map(|row| (row, lane)).collect(),
        Move::Down => (0..size).rev().map(|row| (row, lane)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animated_tiles_include_only_tiles_that_move_or_merge() {
        let board = Board::from_grid([
            [Tile::TWO, Tile::EMPTY, Tile::FOUR, Tile::EMPTY],
            [Tile::TWO, Tile::TWO, Tile::EMPTY, Tile::EMPTY],
            [Tile::EMPTY, Tile::EMPTY, Tile::EMPTY, Tile::EMPTY],
            [Tile::FOUR, Tile::EMPTY, Tile::FOUR, Tile::EMPTY],
        ])
        .expect("board ready");

        let animated = animated_tiles(Move::Left, &board);

        assert_eq!(
            animated,
            vec![
                AnimatedTile {
                    tile: Tile::FOUR,
                    from: (0, 2),
                    to: (0, 1),
                },
                AnimatedTile {
                    tile: Tile::TWO,
                    from: (1, 0),
                    to: (1, 0),
                },
                AnimatedTile {
                    tile: Tile::TWO,
                    from: (1, 1),
                    to: (1, 0),
                },
                AnimatedTile {
                    tile: Tile::FOUR,
                    from: (3, 0),
                    to: (3, 0),
                },
                AnimatedTile {
                    tile: Tile::FOUR,
                    from: (3, 2),
                    to: (3, 0),
                },
            ]
        );
    }

    #[test]
    fn spawned_cells_tracks_new_tile_after_slide_source_board() {
        let old = Board::from_grid([[Tile::TWO, Tile::EMPTY], [Tile::EMPTY, Tile::EMPTY]])
            .expect("board ready");
        let new = Board::from_grid([[Tile::TWO, Tile::EMPTY], [Tile::EMPTY, Tile::FOUR]])
            .expect("board ready");

        assert_eq!(spawned_cells(&old, &new), Some((1, 1)));
    }
}
