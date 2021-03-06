use clap::Clap;
use std::io::{stdin, stdout, Write};
use termion::{clear, color, event::Key, input::TermRead, raw::IntoRawMode};
use twozero48::{Game, Move, Status};

const RESET: color::Fg<color::Reset> = color::Fg(color::Reset);

// Macros for DRY
/// Prints a single block
macro_rules! print_block {
    ($stream: expr, $color: expr, $block: expr) => {
        write!($stream, "{}{}\t{}", $color, $block, RESET).unwrap()
    };
}

/// Allots color to block based on value, then prints it
macro_rules! color_block {
    ($block: expr, $stream: expr) => {
        match $block {
            2 => print_block!($stream, color::Fg(color::Blue), $block),
            4 => print_block!($stream, color::Fg(color::LightBlue), $block),
            8 => print_block!($stream, color::Fg(color::Cyan), $block),
            16 => print_block!($stream, color::Fg(color::LightCyan), $block),
            32 => print_block!($stream, color::Fg(color::Green), $block),
            64 => print_block!($stream, color::Fg(color::LightGreen), $block),
            128 => print_block!($stream, color::Fg(color::Magenta), $block),
            256 => print_block!($stream, color::Fg(color::LightMagenta), $block),
            512 => print_block!($stream, color::Fg(color::Yellow), $block),
            1024 => print_block!($stream, color::Fg(color::LightYellow), $block),
            2048 => print_block!($stream, color::Fg(color::LightRed), $block),
            _ => write!($stream, "{}\t", $block).unwrap(),
        }
    };
}

/// Prints the entire board
macro_rules! print_board {
    ($stream: expr, $board: expr) => {{
        write!($stream, "{}", clear::All).unwrap();

        for row in $board {
            for block in row {
                color_block!(block, $stream)
            }
            writeln!($stream, "\n\r").unwrap();
        }
    }};
}

/// Define the arguments and the CLI option interface for twozero48.
#[derive(Clap)]
#[clap(version = "0.1.3", author = "Devdutt Shenoi <devdutt@outlook.in>")]
struct Opts {
    /// Game board's length & breadth, should be equal to 2 or greater in value,
    /// else it will be automatically updated to the minimum value.
    #[clap(short, long, default_value = "4")]
    pub board_size: usize,
    /// Game's winning point, should be atleast 4 or else a power of 2.
    /// All other user provided options will be automatically updated to the minimum value.
    #[clap(short, long, default_value = "2048")]
    pub winning: usize,
}

fn main() {
    // Collect command line arguments to initiate/configure a game
    let opts = Opts::parse();
    let mut game = Game::new(opts.board_size, opts.winning);

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

        print_board!(stdout, game.board());

        write!(
            stdout,
            "Press A D W S or arrow keys to slide Left Right Up Down\n\rTo win, the board must contain the value: ",
        )
        .unwrap();

        color_block!(game.winning(), stdout);

        writeln!(stdout, "\r").unwrap();

        let mov = match keys.next().unwrap().unwrap() {
            Key::Char('q') | Key::Char('Q') | Key::Ctrl('c') => break,
            Key::Char('a') | Key::Char('A') | Key::Left => Move::Left,
            Key::Char('d') | Key::Char('D') | Key::Right => Move::Right,
            Key::Char('w') | Key::Char('W') | Key::Up => Move::Up,
            Key::Char('s') | Key::Char('S') | Key::Down => Move::Down,
            _ => Move::Dont,
        };

        valid_move = game.mover(mov);
        match game.status() {
            Status::On => continue,
            x => {
                print_board!(stdout, game.board());
                writeln!(
                    stdout,
                    "{}{}{}",
                    color::Fg(color::Red),
                    match x {
                        Status::Lost => "You have lost!",
                        _ => "You have Won!",
                    },
                    RESET
                )
                .unwrap();
                break;
            }
        }
    }
}
