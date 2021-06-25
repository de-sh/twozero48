use rand::prelude::*;
use std::io::{self, Write};

const WINNING: i32 = 2048;
const BOARD_SIZE: usize = 4;
type Board = Vec<Vec<i32>>;

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

/// Performs the compression of board's values towards the left most column
fn move_left(board: &mut Board) {
    for i in 0..BOARD_SIZE {
        vec_compress(&mut board[i]);
    }
}

/// Performs the compression of board's values towards the right most column
fn move_right(board: &mut Board) {
    for i in 0..BOARD_SIZE {
        let mut v = board[i].clone();

        v.reverse();
        vec_compress(&mut v);
        v.reverse();

        board[i] = v;
    }
}

/// Performs the compression of board's values towards the top row
fn move_up(board: &mut Board) {
    for i in 0..BOARD_SIZE {
        let mut v = vec![];
        for j in 0..BOARD_SIZE {
            v.push(board[j][i]);
        }

        vec_compress(&mut v);

        for j in 0..BOARD_SIZE {
            board[j][i] = v[j];
        }
    }
}

/// Performs the compression of board's values towards the bottom row
fn move_down(board: &mut Board) {
    for i in 0..BOARD_SIZE {
        let mut v = vec![];
        for j in 0..BOARD_SIZE {
            v.push(board[j][i]);
        }

        v.reverse();
        vec_compress(&mut v);
        v.reverse();

        for j in 0..BOARD_SIZE {
            board[j][i] = v[j];
        }
    }
}

/// Prints the board as is
fn print(board: &Board) {
    for row in board.iter() {
        for j in row.iter() {
            print!("{} ", j);
        }
        println!();
    }
}

/// Sets a random location to the value 2 if currently 0
fn spawn(board: &mut Board) {
    let mut rng = rand::thread_rng();

    loop {
        let x: usize = rng.gen();
        if board[x % BOARD_SIZE][(x / 10) % BOARD_SIZE] == 0 {
            if x % 5 == 0 {
                board[x % BOARD_SIZE][(x / 10) % BOARD_SIZE] = 4;
            } else {
                board[x % BOARD_SIZE][(x / 10) % BOARD_SIZE] = 2;
            }
            break;
        }
    }
}

/// Verify if board is filled and no valid moves left
fn is_locked(board: &Board) -> bool {
    if contains(&board, 0) {
        return false;
    }

    for i in 0..BOARD_SIZE {
        for j in 0..BOARD_SIZE {
            if i != BOARD_SIZE - 1 {
                if board[i][j] == board[i + 1][j] {
                    return false;
                }
            }
            if j != BOARD_SIZE - 1 {
                if board[i][j] == board[i][j + 1] {
                    return false;
                }
            }
        }
    }

    true
}

/// Check if board contains value x
fn contains(board: &Board, x: i32) -> bool {
    board
        .iter()
        .fold(false, |t, v| t || v.iter().fold(false, |u, w| u || *w == x))
}

/// Used to depict user choice, used as input to API
enum Move {
    Left,
    Right,
    Up,
    Down,
    Dont
}

/// Game entry-point
fn mover(mut board: Board, mov: Move) -> Result<(Board, bool), String> {
    let temp = board.clone();

    match mov {
        Move::Left => move_left(&mut board),
        Move::Right => move_right(&mut board),
        Move::Up => move_up(&mut board),
        Move::Down => move_down(&mut board),
        _ => (),
    }

    if is_locked(&board) {
        return Err("You have Lost!".to_string());
    }

    if contains(&board, WINNING) {
        return Err("You have Won!".to_string());
    }

    let valid_move = board != temp;

    Ok((board, valid_move))
}

fn main() {
    // Initializes the game board
    let mut board = vec![
        vec![0, 0, 0, 0],
        vec![0, 0, 0, 0],
        vec![0, 0, 0, 0],
        vec![0, 0, 0, 0],
    ];
    // Spawns first value
    spawn(&mut board);

    let mut valid_move = true;

    println!("Press A D W S to slide Left Right Up Down\n");

    loop {
        if valid_move {
            spawn(&mut board);
            print(&board);
            print!("\nInput: ");
        } else {
            print!("ILLEGAL INPUT, TRY AGAIN\nInput: ");
        }

        io::stdout().flush().expect("Error");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let mov = match input.chars().nth(0).unwrap().to_ascii_lowercase() {
            'a' => Move::Left,
            'd' => Move::Right,
            'w' => Move::Up,
            's' => Move::Down,
            _ => Move::Dont,
        };

        match mover(board, mov) {
            Ok((b, v)) => {
                board = b;
                valid_move = v;
            },
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };
    }
}
