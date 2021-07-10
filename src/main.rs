use std::io::{stdin, stdout, Stdout, Write};
use termion::{
    clear, color,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};
use twozero48::{Game, Move};

const RESET: color::Fg<color::Reset> = color::Fg(color::Reset);

fn print_board(stdout: &mut RawTerminal<Stdout>, board: &Vec<Vec<i32>>) {
    write!(stdout, "{}", clear::All).unwrap();

    for row in board {
        for block in row {
            // Define a macro for DRY
            macro_rules! print_block {
                ($color: expr) => {
                    write!(stdout, "{}{}\t{}", $color, block, RESET).unwrap()
                };
            }

            match block {
                2 => print_block!(color::Fg(color::Blue)),
                4 => print_block!(color::Fg(color::LightBlue)),
                8 => print_block!(color::Fg(color::Cyan)),
                16 => print_block!(color::Fg(color::LightCyan)),
                32 => print_block!(color::Fg(color::Green)),
                64 => print_block!(color::Fg(color::LightGreen)),
                128 => print_block!(color::Fg(color::Magenta)),
                256 => print_block!(color::Fg(color::LightMagenta)),
                512 => print_block!(color::Fg(color::Yellow)),
                1024 => print_block!(color::Fg(color::LightYellow)),
                2048 => print_block!(color::Fg(color::LightRed)),
                _ => write!(stdout, "{}\t", block).unwrap(),
            }
        }
        writeln!(stdout, "\n\r").unwrap();
    }
}
fn main() {
    // Initializes the game
    macro_rules! input {
        ($msg: expr) => {{
            let mut input = String::new();
            print!("{}", $msg);
            stdout().flush().unwrap();
            stdin().read_line(&mut input).expect("Error reading input");
            input.trim().parse().unwrap()
        }};
    }
    let board_size: usize = input!("Input board size: ");
    let winning: i32 = input!("Input winning number: ");

    let mut game = Game::new(board_size, winning);

    let mut valid_move = true;

    let mut keys = stdin().keys();
    let mut stdout = stdout().into_raw_mode().unwrap();

    loop {
        if valid_move {
            game.refresh();
        } else {
            write!(
                stdout,
                "{}ILLEGAL INPUT, TRY AGAIN{}\n\n\r",
                color::Fg(color::Red),
                RESET
            )
            .unwrap();
        }

        print_board(&mut stdout, game.board());
        writeln!(
            stdout,
            "Press A D W S or arrow keys to slide Left Right Up Down\n\r"
        )
        .unwrap();

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
                print_board(&mut stdout, game.board());
                writeln!(stdout, "{}{}{}", color::Fg(color::Red), e, RESET).unwrap();
                break;
            }
        };
    }
}
