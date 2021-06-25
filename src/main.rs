use std::io::{self, Write};
use twozero48::{Board, Move};

fn main() {
    // Initializes the game board
    let mut board = Board::new();

    let mut valid_move = true;

    println!("Press A D W S to slide Left Right Up Down\n");

    loop {
        if valid_move {
            board.refresh();
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

        match board.mover(mov) {
            Ok(v) => valid_move = v,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };
    }
}
