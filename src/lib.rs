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

type Board = Vec<Vec<usize>>;

/// An object that models the board to play 2048 on and defines the rules for the game
pub struct Game {
    board: Board,
    board_size: usize,
    winning: usize,
}

impl Game {
    /// Constructs a board to play the game
    pub fn new(board_size: usize, winning: usize) -> Self {
        let empty = vec![0].repeat(board_size);
        let mut board = vec![];
        board.resize(board_size, empty);

        // Ensure the winning point is a power of two, in order to be reached within game logic
        let winning = 2usize.pow((winning as f32).log2().abs() as u32);

        let mut init = Self {
            board,
            board_size,
            winning,
        };

        // Spawns first random value
        init.spawn();

        init
    }

    /// Return immutable reference to the board
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Returns the value for winning
    pub fn winning(&self) -> usize {
        self.winning
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
            let mut v = vec![];
            for j in 0..self.board_size {
                v.push(self.board[j][i]);
            }

            self.vec_compress(&mut v);

            for j in 0..self.board_size {
                self.board[j][i] = v[j];
            }
        }
    }

    /// Performs the compression of board's values towards the bottom row
    fn move_down(&mut self) {
        for i in 0..self.board_size {
            let mut v = vec![];
            for j in 0..self.board_size {
                v.push(self.board[j][i]);
            }

            v.reverse();
            self.vec_compress(&mut v);
            v.reverse();

            for j in 0..self.board_size {
                self.board[j][i] = v[j];
            }
        }
    }

    /// Sets a random location to the value 2 if currently 0
    fn spawn(&mut self) {
        let mut rng = rand::thread_rng();

        loop {
            let x: usize = rng.gen();
            if self.board[x % self.board_size][(x / 10) % self.board_size] == 0 {
                if x % 5 == 0 {
                    self.board[x % self.board_size][(x / 10) % self.board_size] = 4;
                } else {
                    self.board[x % self.board_size][(x / 10) % self.board_size] = 2;
                }
                break;
            }
        }
    }

    /// To refresh and return a reference to the game board after a valid move
    pub fn refresh(&mut self) {
        self.spawn();
    }

    /// Verify if board is filled and no valid moves left
    fn is_locked(&self) -> bool {
        if self.contains(0) {
            return false;
        }

        for i in 0..self.board_size {
            for j in 0..self.board_size {
                if i != self.board_size - 1 {
                    if self.board[i][j] == self.board[i + 1][j] {
                        return false;
                    }
                }
                if j != self.board_size - 1 {
                    if self.board[i][j] == self.board[i][j + 1] {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Check if board contains value x
    fn contains(&self, x: usize) -> bool {
        self.board
            .iter()
            .fold(false, |t, v| t || v.iter().fold(false, |u, w| u || *w == x))
    }

    /// [`Game`] API entry-point, operated by [`Move`] as input
    pub fn mover(&mut self, mov: Move) -> Result<bool, String> {
        let temp = self.board.clone();

        match mov {
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::Up => self.move_up(),
            Move::Down => self.move_down(),
            _ => (),
        }

        if self.is_locked() {
            return Err("You have Lost!".to_string());
        }

        if self.contains(self.winning) {
            return Err("You have Won!".to_string());
        }

        Ok(self.board != temp)
    }

    /// Compress a row/column
    fn vec_compress(&self, v: &mut Vec<usize>) {
        v.retain(|x| *x != 0);
        let vl = v.len();

        if vl > 1 {
            for i in 0..vl - 1 {
                if v[i] == v[i + 1] {
                    v[i] *= 2;
                    v[i + 1] = 0;
                }
            }
        }

        v.retain(|x| *x != 0);
        v.resize(self.board_size, 0);
    }
}
