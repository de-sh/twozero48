use std::fmt::Display;

use rand::prelude::*;

mod board;
mod tui;

pub use board::Board;
pub use tui::State;

/// Directional input to the [`Game`] API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    /// Executes leftward compression of board elements
    Left,
    /// Executes rightward compression of board elements
    Right,
    /// Executes upward compression of board elements
    Up,
    /// Executes downward compression of board elements
    Down,
}

/// Used to depict the status in the [`Game`] API
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Game has finished, player won
    Won,
    /// Game has finished, player lost
    Lost,
    /// Game continues, neither won nor lost
    On,
}

pub trait Play {
    fn board(&self) -> &Board;
    fn winning(&self) -> Tile;
    fn score(&self) -> usize;
    fn largest_tile(&self) -> Tile;
    fn status(&self) -> Status;
    fn mover(&mut self, mov: Move) -> bool;
    fn spawn(&mut self);
}

/// Represents a tile on the game board. exp is the exponent of 2 that the tile represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tile {
    exp: u32,
}

impl Tile {
    pub const EMPTY: Tile = Tile { exp: 0 };
    pub const TWO: Tile = Tile { exp: 1 };
    pub const FOUR: Tile = Tile { exp: 2 };
    pub const ONE_TWO_EIGHT: Tile = Tile { exp: 7 };

    pub const fn new(exp: u32) -> Self {
        Tile { exp }
    }

    /// Returns the score value of the tile.
    pub fn score(&self) -> usize {
        // Score is 0 for empty tiles
        if self.exp == 0 {
            return 0;
        }

        // Score is 2^exp for other tiles
        1 << self.exp
    }

    /// Returns the exponent of the tile.
    pub fn exponent(&self) -> u32 {
        self.exp
    }

    /// Doubles the tile value.
    fn promote(self) -> Self {
        Tile { exp: self.exp + 1 }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.score())
    }
}

/// An object that models the board to play 2048 on and defines the rules for the game
pub struct Game {
    board: Board,
    winning: Tile,
    score: usize,
}

impl Game {
    /// Constructs a board to play the game
    /// board_size >= 2, defines board's length & breadth
    /// winning defines the Tile for the game to have been won
    pub fn new(board_size: usize, winning: Tile) -> Result<Self, String> {
        let mut init = Self {
            board: Board::new(board_size)?,
            winning,
            score: 0,
        };

        // Spawns first random value
        init.spawn();

        Ok(init)
    }
}

impl Play for Game {
    /// Return immutable reference to the board
    fn board(&self) -> &Board {
        &self.board
    }

    /// Returns the Tile for winning
    fn winning(&self) -> Tile {
        self.winning
    }

    /// Returns the current score
    fn score(&self) -> usize {
        self.score
    }

    /// Returns the current largest tile on the board
    fn largest_tile(&self) -> Tile {
        self.board.max_tile()
    }

    /// Returns the current status of the game
    fn status(&self) -> Status {
        if self.board.contains(self.winning) {
            Status::Won
        } else if self.board.is_locked() {
            Status::Lost
        } else {
            Status::On
        }
    }

    /// [`Game`] API entry-point, operated by [`Move`] as input
    /// Output bool is used to check if move caused any change to the board
    fn mover(&mut self, mov: Move) -> bool {
        let temp = self.board.clone();

        self.score += match mov {
            Move::Left => self.board.move_left(),
            Move::Right => self.board.move_right(),
            Move::Up => self.board.move_up(),
            Move::Down => self.board.move_down(),
        };

        self.board != temp
    }

    /// Sets a random empty cell to 2 (90%) or 4 (10%). No-op if board is full.
    fn spawn(&mut self) {
        let empty: Vec<(usize, usize)> = (0..self.board.size())
            .flat_map(|r| (0..self.board.size()).map(move |c| (r, c)))
            .filter(|&(r, c)| self.board[(r, c)] == Tile::EMPTY)
            .collect();
        if empty.is_empty() {
            return;
        }
        let mut rng = rand::rng();
        let (r, c) = empty[rng.random_range(0..empty.len())];
        self.board[(r, c)] = if rng.random_bool(0.1) {
            Tile::FOUR
        } else {
            Tile::TWO
        };
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn score_starts_at_zero() {
        let game = Game::new(4, Tile { exp: 11 }).expect("game created");
        assert_eq!(game.score(), 0);
    }

    #[test]
    fn score_accumulates_after_merge() {
        let mut game = Game::new(2, Tile { exp: 11 }).expect("game created");
        // Force a known board state: [[2,2], [0,0]]
        game.board = Board::from_grid([[Tile::TWO, Tile::TWO], [Tile::EMPTY, Tile::EMPTY]])
            .expect("board ready");
        game.mover(Move::Left);
        assert_eq!(game.score(), 4);
    }

    #[test]
    fn current_largest_tile_on_board() {
        let mut game = Game::new(4, Tile { exp: 11 }).expect("game created");
        game.board[(0, 0)] = Tile { exp: 6 };
        assert_eq!(game.largest_tile(), Tile { exp: 6 });
    }
}
