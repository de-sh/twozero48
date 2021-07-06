use std::io::{self, Write};
use twozero48::{Game, Move};


fn main() {
    // Initializes the game
    let mut game = Game::new(4, 2048);

    let mut valid_move = true;

    println!("Press A D W S to slide Left Right Up Down\n");

    loop {
        if valid_move {
            for row in game.refreshed() {
                for block in row {
                    print!("{} ", block);
                }
                println!();
            }
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

        valid_move = match game.mover(mov) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        };
    }
}
