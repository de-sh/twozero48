use std::io::{stdin, stdout, Write};
use termion::{event::Key, color, clear, input::TermRead, raw::IntoRawMode};
use twozero48::{Game, Move};

fn main() {
    let mut keys = stdin().keys();
    let mut stdout = stdout().into_raw_mode().unwrap();

    // Initializes the game
    let mut game = Game::new(4, 2048);

    let mut valid_move = true;
    let reset = color::Fg(color::Reset);

    println!("Press A D W S or arrow keys to slide Left Right Up Down\n\r");

    loop {
        write!(stdout, "{}", clear::All).unwrap();
        if valid_move {
            for row in game.refreshed() {
                for block in row {
                    match block {
                        2 => write!(stdout, "{}{}\t{}", color::Fg(color::Blue), block, reset).unwrap(),
                        4 => write!(stdout, "{}{}\t{}", color::Fg(color::LightBlue), block, reset).unwrap(),
                        8 => write!(stdout, "{}{}\t{}", color::Fg(color::Cyan), block, reset).unwrap(),
                        16 => write!(stdout, "{}{}\t{}", color::Fg(color::LightCyan), block, reset).unwrap(),
                        32 => write!(stdout, "{}{}\t{}", color::Fg(color::Green), block, reset).unwrap(),
                        64 => write!(stdout, "{}{}\t{}", color::Fg(color::LightGreen), block, reset).unwrap(),
                        128 => write!(stdout, "{}{}\t{}", color::Fg(color::Magenta), block, reset).unwrap(),
                        256 => write!(stdout, "{}{}\t{}", color::Fg(color::LightMagenta), block, reset).unwrap(),
                        512 => write!(stdout, "{}{}\t{}", color::Fg(color::Red), block, reset).unwrap(),
                        1024 => write!(stdout, "{}{}\t{}", color::Fg(color::LightRed), block, reset).unwrap(),
                        2048 => write!(stdout, "{}{}\t{}", color::Fg(color::Yellow), block, reset).unwrap(),
                        _ => write!(stdout, "{}\t", block).unwrap(),
                    }
                }
                writeln!(stdout, "\n\r").unwrap();
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
