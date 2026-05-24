use std::fmt::Display;

use rand::prelude::*;

/// Used to depict user choice, an input to the [`Game`] API
#[derive(Clone, Copy)]
pub enum Move {
    /// Executes leftward compression of board elements
    Left,
    /// Executes rightward compression of board elements
    Right,
    /// Executes upward compression of board elements
    Up,
    /// Executes downward compression of board elements
    Down,
    /// Condition triggered incase input is improper
    Dont,
}

/// Used to depict the status in the [`Game`] API
pub enum Status {
    /// Game has finished, player won
    Won,
    /// Game has finished, player lost
    Lost,
    /// Game continues, neither won nor lost
    On,
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

pub type Board = Vec<Vec<Tile>>;

/// An object that models the board to play 2048 on and defines the rules for the game
pub struct Game {
    board: Board,
    board_size: usize,
    winning: Tile,
    score: usize,
}

impl Game {
    /// Constructs a board to play the game
    /// board_size >= 2, defines board's length & breadth
    /// winning defines the Tile for the game to have been won
    pub fn new(board_size: usize, winning: Tile) -> Self {
        // Ensure the board size is at least 2
        let board_size = board_size.clamp(2, usize::MAX);
        let board = vec![vec![Tile::EMPTY; board_size]; board_size];

        let mut init = Self {
            board,
            board_size,
            winning,
            score: 0,
        };

        // Spawns first random value
        init.spawn();

        init
    }

    /// Return immutable reference to the board
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Returns the Tile for winning
    pub fn winning(&self) -> Tile {
        self.winning
    }

    /// Returns the current score
    pub fn score(&self) -> usize {
        self.score
    }

    /// Performs the compression of board's values towards the left most column
    fn move_left(&mut self) {
        for i in 0..self.board_size {
            let mut v = self.board[i].clone();

            self.vec_compress(&mut v);

            self.board[i] = v;
        }
    }

    /// Performs the compression of board's values towards the right most column
    fn move_right(&mut self) {
        for i in 0..self.board_size {
            let mut v = self.board[i].clone();

            v.reverse();
            self.vec_compress(&mut v);
            v.reverse();

            self.board[i] = v;
        }
    }

    /// Performs the compression of board's values towards the top row
    fn move_up(&mut self) {
        for i in 0..self.board_size {
            let mut v = (0..self.board_size).map(|j| self.board[j][i]).collect();

            self.vec_compress(&mut v);

            for (j, tile) in v.iter().enumerate().take(self.board_size) {
                self.board[j][i] = *tile;
            }
        }
    }

    /// Performs the compression of board's values towards the bottom row
    fn move_down(&mut self) {
        for i in 0..self.board_size {
            let mut v: Vec<Tile> = (0..self.board_size).map(|j| self.board[j][i]).collect();

            v.reverse();
            self.vec_compress(&mut v);
            v.reverse();

            for (j, tile) in v.iter().enumerate().take(self.board_size) {
                self.board[j][i] = *tile;
            }
        }
    }

    /// Sets a random empty cell to 2 (90%) or 4 (10%). No-op if board is full.
    fn spawn(&mut self) {
        let empty: Vec<(usize, usize)> = (0..self.board_size)
            .flat_map(|r| (0..self.board_size).map(move |c| (r, c)))
            .filter(|&(r, c)| self.board[r][c] == Tile::EMPTY)
            .collect();
        if empty.is_empty() {
            return;
        }
        let mut rng = rand::rng();
        let (r, c) = empty[rng.random_range(0..empty.len())];
        self.board[r][c] = if rng.random_bool(0.1) {
            Tile::FOUR
        } else {
            Tile::TWO
        };
    }

    /// Returns the current largest tile on the board
    pub fn largest_tile(&self) -> Tile {
        self.board()
            .iter()
            .flatten()
            .copied()
            .max()
            .unwrap_or(Tile::EMPTY)
    }

    /// Refreshes(spawns new tile on an empty cell) the board after a valid move
    pub fn refresh(&mut self) {
        self.spawn();
    }

    /// Verify if board is filled and no valid moves left
    fn is_locked(&self) -> bool {
        if self.contains(Tile::EMPTY) {
            return false;
        }

        for i in 0..self.board_size {
            for j in 0..self.board_size {
                if i != self.board_size - 1 && self.board[i][j] == self.board[i + 1][j] {
                    return false;
                }
                if j != self.board_size - 1 && self.board[i][j] == self.board[i][j + 1] {
                    return false;
                }
            }
        }

        true
    }

    /// Check if board contains value x
    fn contains(&self, x: Tile) -> bool {
        self.board.iter().any(|v| v.contains(&x))
    }

    pub fn status(&self) -> Status {
        if self.contains(self.winning) {
            Status::Won
        } else if self.is_locked() {
            Status::Lost
        } else {
            Status::On
        }
    }

    /// [`Game`] API entry-point, operated by [`Move`] as input
    /// Output bool is used to check if move caused any change to the board
    pub fn mover(&mut self, mov: Move) -> bool {
        let temp = self.board.clone();

        match mov {
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::Up => self.move_up(),
            Move::Down => self.move_down(),
            _ => (),
        }

        self.board != temp
    }

    /// Compress a row/column, keeps track of score earned from merges
    fn vec_compress(&mut self, v: &mut Vec<Tile>) {
        v.retain(|x| *x != Tile::EMPTY);
        let vl = v.len();

        if vl > 1 {
            for i in 0..vl - 1 {
                if v[i] == v[i + 1] {
                    let promoted = v[i].promote();
                    v[i] = promoted;
                    v[i + 1] = Tile::EMPTY;
                    self.score += promoted.score();
                }
            }
        }

        v.retain(|x| *x != Tile::EMPTY);
        v.resize(self.board_size, Tile::EMPTY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_compress_no_merge() {
        let mut v = vec![Tile::TWO, Tile::FOUR, Tile { exp: 3 }, Tile::EMPTY];
        let mut game = Game::new(4, Tile::FOUR);
        game.vec_compress(&mut v);
        assert_eq!(v, vec![Tile::TWO, Tile::FOUR, Tile { exp: 3 }, Tile::EMPTY]);
    }

    #[test]
    fn vec_compress_single_merge() {
        let mut v = vec![Tile::TWO, Tile::TWO, Tile::EMPTY, Tile::EMPTY];
        let mut game = Game::new(4, Tile::FOUR);
        game.vec_compress(&mut v);
        assert_eq!(v, vec![Tile::FOUR, Tile::EMPTY, Tile::EMPTY, Tile::EMPTY]);
    }

    #[test]
    fn vec_compress_multiple_merges() {
        let mut v = vec![Tile::FOUR, Tile::FOUR, Tile::FOUR, Tile::FOUR];
        let mut game = Game::new(4, Tile::FOUR);
        game.vec_compress(&mut v);
        assert_eq!(
            v,
            vec![Tile { exp: 3 }, Tile::EMPTY, Tile::EMPTY, Tile::EMPTY]
        );
    }

    #[test]
    fn vec_compress_no_double_merge() {
        // [2,2,2,0] → [4,2,0,0]: only first pair merges, score=4
        let mut v = vec![Tile::TWO, Tile::TWO, Tile::TWO, Tile::EMPTY];
        let mut game = Game::new(4, Tile::FOUR);
        game.vec_compress(&mut v);
        assert_eq!(v, vec![Tile::FOUR, Tile::TWO, Tile::EMPTY, Tile::EMPTY]);
    }

    #[test]
    fn score_starts_at_zero() {
        let game = Game::new(4, Tile { exp: 11 });
        assert_eq!(game.score(), 0);
    }

    #[test]
    fn score_accumulates_after_merge() {
        let mut game = Game::new(2, Tile { exp: 11 });
        // Force a known board state: [2,2] / [0,0]
        game.board[0] = vec![Tile::TWO, Tile::TWO];
        game.board[1] = vec![Tile::EMPTY, Tile::EMPTY];
        game.mover(Move::Left);
        assert_eq!(game.score(), 4);
    }

    #[test]
    fn current_largest_tile_on_board() {
        let mut game = Game::new(4, Tile { exp: 11 });
        game.board[0][0] = Tile { exp: 6 };
        assert_eq!(game.largest_tile(), Tile { exp: 6 });
    }
}
