use std::{error::Error, time::Duration};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use tui::{MoveEffects, TermGuard};
use twozero48::{Game, Move, Status, Tile};

use crate::milestones::MilestoneChecker;

mod milestones;
mod tui;

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
        1024 => Ok(Tile::OneThousandTwentyFour),
        2048 => Ok(Tile::TwoThousandFourtyEight),
        4096 => Ok(Tile::FourHundredNinetySix),
        _ => Err(format!(
            "{score} is not a supported winning value. Only 128, 256, 512, 1024, 2048, and 4096 are supported values."
        )),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Collect command line arguments to initiate/configure a game
    let opts = Opts::parse();
    let mut game = Game::new(opts.board_size, opts.winning);
    let mut milestone_checker = MilestoneChecker::new(Tile::Empty);

    let mut terminal = TermGuard::new()?;

    let mut valid_move = true;
    let mut move_effects = MoveEffects::new();

    loop {
        move_effects.tick();
        let (xs, ys) = move_effects.shift();

        let message = if !valid_move {
            Some("No tiles moved — try a different direction")
        } else {
            None
        };
        terminal.render_board(&game, message, xs, ys, move_effects.flash())?;

        // Non-blocking poll while animating, blocking otherwise
        if move_effects.is_active() && !event::poll(Duration::from_millis(30))? {
            continue;
        }

        let event = event::read()?;
        // Non-key events (resize, focus, mouse) should not preserve the stale
        // "No tiles moved" message from a previous invalid directional input.
        if !matches!(event, Event::Key(_)) {
            valid_move = true;
            continue;
        }
        if let Event::Key(key) = event {
            let mov = match (key.code, key.modifiers) {
                (KeyCode::Char('q'), _)
                | (KeyCode::Char('Q'), _)
                | (KeyCode::Char('c'), KeyModifiers::CONTROL)
                | (KeyCode::Esc, _) => break,
                (KeyCode::Char('a'), _) | (KeyCode::Char('A'), _) | (KeyCode::Left, _) => {
                    Move::Left
                }
                (KeyCode::Char('d'), _) | (KeyCode::Char('D'), _) | (KeyCode::Right, _) => {
                    Move::Right
                }
                (KeyCode::Char('w'), _) | (KeyCode::Char('W'), _) | (KeyCode::Up, _) => Move::Up,
                (KeyCode::Char('s'), _) | (KeyCode::Char('S'), _) | (KeyCode::Down, _) => {
                    Move::Down
                }
                _ => Move::Dont,
            };

            let old_board = game.board().clone();
            valid_move = game.mover(mov);

            if valid_move {
                move_effects.record_move(mov, &old_board, game.board());

                game.refresh();

                let current_largest = game.largest_tile();
                if milestone_checker.is_milestone(current_largest) {
                    terminal.render_board(
                        &game,
                        Some(&format!("{current_largest} reached!")),
                        0,
                        0,
                        move_effects.flash(),
                    )?;
                }
            }

            match game.status() {
                Status::On => continue,
                status => {
                    let end_msg = match status {
                        Status::Won => "You won!  Press any key to exit.",
                        Status::Lost => "Game over!  Press any key to exit.",
                        Status::On => unreachable!(),
                    };
                    move_effects.clear();
                    terminal.render_board(&game, Some(end_msg), 0, 0, move_effects.flash())?;
                    event::read()?;
                    break;
                }
            }
        }
    }

    Ok(())
}
