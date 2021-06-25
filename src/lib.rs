use rand::prelude::*;

const WINNING: i32 = 2048;
const BOARD_SIZE: usize = 4;

/// Compress a row/column
fn vec_compress(v: &mut Vec<i32>) {
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
    v.resize(BOARD_SIZE, 0);
}

/// Used to depict user choice, used as input to API
pub enum Move {
    Left,
    Right,
    Up,
    Down,
    Dont,
}

pub struct Board(Vec<Vec<i32>>);

impl Board {
    pub fn new() -> Self {
        let mut init = Self(vec![
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ]);
        // Spawns first random value
        init.spawn();

        init
    }

    /// Performs the compression of board's values towards the left most column
    fn move_left(&mut self) {
        for i in 0..BOARD_SIZE {
            vec_compress(&mut self.0[i]);
        }
    }

    /// Performs the compression of board's values towards the right most column
    fn move_right(&mut self) {
        for i in 0..BOARD_SIZE {
            let mut v = self.0[i].clone();

            v.reverse();
            vec_compress(&mut v);
            v.reverse();

            self.0[i] = v;
        }
    }

    /// Performs the compression of board's values towards the top row
    fn move_up(&mut self) {
        for i in 0..BOARD_SIZE {
            let mut v = vec![];
            for j in 0..BOARD_SIZE {
                v.push(self.0[j][i]);
            }

            vec_compress(&mut v);

            for j in 0..BOARD_SIZE {
                self.0[j][i] = v[j];
            }
        }
    }

    /// Performs the compression of board's values towards the bottom row
    fn move_down(&mut self) {
        for i in 0..BOARD_SIZE {
            let mut v = vec![];
            for j in 0..BOARD_SIZE {
                v.push(self.0[j][i]);
            }

            v.reverse();
            vec_compress(&mut v);
            v.reverse();

            for j in 0..BOARD_SIZE {
                self.0[j][i] = v[j];
            }
        }
    }

    /// Prints the board as is
    pub fn print(&self) {
        for row in self.0.iter() {
            for j in row.iter() {
                print!("{} ", j);
            }
            println!();
        }
    }

    /// Sets a random location to the value 2 if currently 0
    fn spawn(&mut self) {
        let mut rng = rand::thread_rng();

        loop {
            let x: usize = rng.gen();
            if self.0[x % BOARD_SIZE][(x / 10) % BOARD_SIZE] == 0 {
                if x % 5 == 0 {
                    self.0[x % BOARD_SIZE][(x / 10) % BOARD_SIZE] = 4;
                } else {
                    self.0[x % BOARD_SIZE][(x / 10) % BOARD_SIZE] = 2;
                }
                break;
            }
        }
    }

    /// To refresh a board after a valid move
    pub fn refresh(&mut self) {
        self.spawn();
        self.print();
    }

    /// Verify if board is filled and no valid moves left
    fn is_locked(&self) -> bool {
        if self.contains(0) {
            return false;
        }

        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                if i != BOARD_SIZE - 1 {
                    if self.0[i][j] == self.0[i + 1][j] {
                        return false;
                    }
                }
                if j != BOARD_SIZE - 1 {
                    if self.0[i][j] == self.0[i][j + 1] {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Check if board contains value x
    fn contains(&self, x: i32) -> bool {
        self.0
            .iter()
            .fold(false, |t, v| t || v.iter().fold(false, |u, w| u || *w == x))
    }

    /// Game entry-point
    pub fn mover(&mut self, mov: Move) -> Result<bool, String> {
        let temp = self.0.clone();

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

        if self.contains(WINNING) {
            return Err("You have Won!".to_string());
        }

        Ok(self.0 != temp)
    }
}
