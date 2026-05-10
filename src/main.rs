use std::io::{Write, stdin, stdout};

use clap::Parser;
use termion::{clear, color, event::Key, input::TermRead, raw::IntoRawMode};
use twozero48::{Game, Move, Status, Tile};

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
            Tile::Two => print_block!($stream, color::Fg(color::Blue), $block),
            Tile::Four => print_block!($stream, color::Fg(color::LightBlue), $block),
            Tile::Eight => print_block!($stream, color::Fg(color::Cyan), $block),
            Tile::Sixteen => print_block!($stream, color::Fg(color::LightCyan), $block),
            Tile::ThirtyTwo => print_block!($stream, color::Fg(color::Green), $block),
            Tile::SixtyFour => print_block!($stream, color::Fg(color::LightGreen), $block),
            Tile::OneHundredTwentyEight => print_block!($stream, color::Fg(color::Magenta), $block),
            Tile::TwoHundredFiftySix => {
                print_block!($stream, color::Fg(color::LightMagenta), $block)
            }
            Tile::FiveHundredTwelve => print_block!($stream, color::Fg(color::Yellow), $block),
            Tile::OneThousandTwoFour => {
                print_block!($stream, color::Fg(color::LightYellow), $block)
            }
            Tile::TwoThousandFourEight => print_block!($stream, color::Fg(color::LightRed), $block),
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
#[derive(Parser)]
#[clap(
    version = clap::crate_version!(),
    author = "Devdutt Shenoi <devdutt@outlook.in>"
)]
struct Opts {
    /// Game board's length & breadth, should be equal to 2 or greater in value,
    /// else it will be automatically updated to the minimum value.
    #[clap(short, long, default_value = "4")]
    pub board_size: usize,
    /// Game's winning tile value, only 128, 256, 512, 1024, 2048, and 4096 are supported values.
    /// All other user provided options will be automatically updated to the minimum value.
    #[clap(short, long, default_value = "2048", value_parser = parse_winning)]
    pub winning: Tile,
}

fn parse_winning(score: &str) -> Result<Tile, String> {
    let score: usize = score
        .parse()
        .map_err(|_| "Only 128, 256, 512, 1024, 2048, and 4096 are supported values")?;
    match score {
        128 => Ok(Tile::OneHundredTwentyEight),
        256 => Ok(Tile::TwoHundredFiftySix),
        512 => Ok(Tile::FiveHundredTwelve),
        1024 => Ok(Tile::OneThousandTwoFour),
        2048 => Ok(Tile::TwoThousandFourEight),
        4096 => Ok(Tile::FourHundredNinetySix),
        _ => Err(format!(
            "{score} is not a supported winning value. Only 128, 256, 512, 1024, 2048, and 4096 are supported values."
        )),
    }
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
