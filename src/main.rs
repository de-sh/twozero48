use std::io::{stdin, stdout, Write};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};
use twozero48::{Game, Move};

fn main() {
    let mut keys = stdin().keys();
    let mut stdout = stdout().into_raw_mode().unwrap();

    // Initializes the game
    let mut game = Game::new(4, 2048);

    let mut valid_move = true;

    println!("Press A D W S or arrow keys to slide Left Right Up Down\n\r");

    loop {
        if valid_move {
            for row in game.refreshed() {
                for block in row {
                    write!(stdout, "{}  ", block).unwrap();
                }
                writeln!(stdout, "\r").unwrap();
            }
        } else {
            write!(stdout, "ILLEGAL INPUT, TRY AGAIN\n\r").unwrap();
        }

        writeln!(stdout, "\r").unwrap();

        let mov = match keys.next().unwrap().unwrap() {
            Key::Char('q') | Key::Char('Q') | Key::Ctrl('c') => break,
            Key::Char('a') | Key::Char('A') | Key::Left => Move::Left,
            Key::Char('d') | Key::Char('D') | Key::Right => Move::Right,
            Key::Char('w') | Key::Char('W') | Key::Up => Move::Up,
            Key::Char('s') | Key::Char('S') | Key::Down => Move::Down,
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
