use rand::prelude::*;
use std::io::{self, Read, Write};

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
fn is_locked(board: &Vec<Vec<i32>>) -> bool {
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
fn contains(board: &Vec<Vec<i32>>, x: i32) -> bool {
    board
        .iter()
        .fold(false, |t, v| t || v.iter().fold(false, |u, w| u || *w == x))
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
        }

        let temp = board.clone();

        io::stdout().flush().expect("Error");

        let input = std::io::stdin()
            .bytes()
            .next()
            .and_then(|result| result.ok())
            .map(|byte| byte as char)
            .unwrap();

        match input.to_ascii_lowercase() {
            'a' => {
                move_left(&mut board);
            }
            'd' => {
                move_right(&mut board);
            }
            'w' => {
                move_up(&mut board);
            }
            's' => {
                move_down(&mut board);
            }
            _ => (),
        }

        valid_move = board != temp;

        if is_locked(&board) {
            println!("You have Lost!");
        }

        if contains(&board, WINNING) {
            println!("You have Won!");
        }
    }
}
