use std::fmt::Display;

use rand::prelude::*;

/// Used to depict user choice, an input to the [`Game`] API
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

/// Represents a tile(value) on the game board.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tile {
    #[default]
    Empty,
    Two,
    Four,
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
    OneHundredTwentyEight,
    TwoHundredFiftySix,
    FiveHundredTwelve,
    OneThousandTwentyFour,
    TwoThousandFourtyEight,
    FourHundredNinetySix,
}

impl Tile {
    /// Returns the score value of the tile.
    fn score(&self) -> usize {
        match self {
            Tile::Empty => 0,
            Tile::Two => 2,
            Tile::Four => 4,
            Tile::Eight => 8,
            Tile::Sixteen => 16,
            Tile::ThirtyTwo => 32,
            Tile::SixtyFour => 64,
            Tile::OneHundredTwentyEight => 128,
            Tile::TwoHundredFiftySix => 256,
            Tile::FiveHundredTwelve => 512,
            Tile::OneThousandTwentyFour => 1024,
            Tile::TwoThousandFourtyEight => 2048,
            Tile::FourHundredNinetySix => 4096,
        }
    }

    /// Promotes the tile to the next value, e.g. `Two` becomes `Four`, `Four` becomes `Eight`, etc.
    /// Empty and 4096 tiles don't change as they are upper limits.
    fn promote(&self) -> Self {
        match self {
            Tile::Empty => Tile::Empty,
            Tile::Two => Tile::Four,
            Tile::Four => Tile::Eight,
            Tile::Eight => Tile::Sixteen,
            Tile::Sixteen => Tile::ThirtyTwo,
            Tile::ThirtyTwo => Tile::SixtyFour,
            Tile::SixtyFour => Tile::OneHundredTwentyEight,
            Tile::OneHundredTwentyEight => Tile::TwoHundredFiftySix,
            Tile::TwoHundredFiftySix => Tile::FiveHundredTwelve,
            Tile::FiveHundredTwelve => Tile::OneThousandTwentyFour,
            Tile::OneThousandTwentyFour => Tile::TwoThousandFourtyEight,
            Tile::TwoThousandFourtyEight => Tile::FourHundredNinetySix,
            Tile::FourHundredNinetySix => Tile::FourHundredNinetySix,
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.score())
    }
}

type Board = Vec<Vec<Tile>>;

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

        // initialize an empty board of 0s
        let empty = [Tile::Empty].repeat(board_size);
        let mut board = vec![];
        board.resize(board_size, empty);

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
            .filter(|&(r, c)| self.board[r][c] == Tile::Empty)
            .collect();
        if empty.is_empty() {
            return;
        }
        let mut rng = rand::rng();
        let (r, c) = empty[rng.random_range(0..empty.len())];
        self.board[r][c] = if rng.random_bool(0.1) {
            Tile::Four
        } else {
            Tile::Two
        };
    }

    /// Returns the current largest tile on the board
    pub fn largest_tile(&self) -> Tile {
        self.board()
            .iter()
            .flatten()
            .copied()
            .max()
            .unwrap_or(Tile::Empty)
    }

    /// To refresh and return a reference to the game board after a valid move
    pub fn refresh(&mut self) {
        self.spawn();
    }

    /// Verify if board is filled and no valid moves left
    fn is_locked(&self) -> bool {
        if self.contains(Tile::Empty) {
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
        v.retain(|x| *x != Tile::Empty);
        let vl = v.len();

        if vl > 1 {
            for i in 0..vl - 1 {
                if v[i] == v[i + 1] {
                    let promoted = v[i].promote();
                    v[i] = promoted;
                    v[i + 1] = Tile::Empty;
                    self.score += promoted.score();
                }
            }
        }

        v.retain(|x| *x != Tile::Empty);
        v.resize(self.board_size, Tile::Empty);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_compress_no_merge() {
        let mut v = vec![Tile::Two, Tile::Four, Tile::Eight, Tile::Empty];
        let mut game = Game::new(4, Tile::Four);
        game.vec_compress(&mut v);
        assert_eq!(v, vec![Tile::Two, Tile::Four, Tile::Eight, Tile::Empty]);
    }

    #[test]
    fn vec_compress_single_merge() {
        let mut v = vec![Tile::Two, Tile::Two, Tile::Empty, Tile::Empty];
        let mut game = Game::new(4, Tile::Four);
        game.vec_compress(&mut v);
        assert_eq!(v, vec![Tile::Four, Tile::Empty, Tile::Empty, Tile::Empty]);
    }

    #[test]
    fn vec_compress_multiple_merges() {
        let mut v = vec![Tile::Four, Tile::Four, Tile::Four, Tile::Four];
        let mut game = Game::new(4, Tile::Four);
        game.vec_compress(&mut v);
        assert_eq!(v, vec![Tile::Eight, Tile::Eight, Tile::Empty, Tile::Empty]);
    }

    #[test]
    fn vec_compress_no_double_merge() {
        // [2,2,2,0] → [4,2,0,0]: only first pair merges, score=4
        let mut v = vec![Tile::Two, Tile::Two, Tile::Two, Tile::Empty];
        let mut game = Game::new(4, Tile::Four);
        game.vec_compress(&mut v);
        assert_eq!(v, vec![Tile::Four, Tile::Two, Tile::Empty, Tile::Empty]);
    }

    #[test]
    fn score_starts_at_zero() {
        let game = Game::new(4, Tile::TwoThousandFourtyEight);
        assert_eq!(game.score(), 0);
    }

    #[test]
    fn score_accumulates_after_merge() {
        let mut game = Game::new(2, Tile::TwoThousandFourtyEight);
        // Force a known board state: [2,2] / [0,0]
        game.board[0] = vec![Tile::Two, Tile::Two];
        game.board[1] = vec![Tile::Empty, Tile::Empty];
        game.mover(Move::Left);
        assert_eq!(game.score(), 4);
    }

    #[test]
    fn current_largest_tile_on_board() {
        let mut game = Game::new(4, Tile::TwoThousandFourtyEight);
        game.board[0][0] = Tile::SixtyFour;
        assert_eq!(game.largest_tile(), Tile::SixtyFour);
    }
}
