use std::{error::Error, time::Duration};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use twozero48::{Game, Move, Status, Tile};

use crate::state::State;

mod state;
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
    /// Game's winning tile value, must be a power of 2 (e.g. 128, 256, 512, 1024, 2048, 4096, ...).
    #[clap(short, long, default_value = "2048", value_parser = parse_winning)]
    pub winning: Tile,
}

fn parse_winning(score: &str) -> Result<Tile, String> {
    let value: usize = score
        .parse()
        .map_err(|_| format!("{score} is not valid; must be a positive number"))?;

    if value.is_power_of_two() && value >= 2 {
        Ok(Tile::new(value.trailing_zeros()))
    } else {
        Err(format!(
            "{value} is not valid; must be a power of 2 (e.g. 2048)"
        ))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Collect command line arguments to initiate/configure a game
    let opts = Opts::parse();
    let game = Game::new(opts.board_size, opts.winning);
    let mut state = State::new(game)?;

    loop {
        state.tick_effects();
        state.render_board()?;

        // Non-blocking poll while animating, blocking otherwise
        if state.effects_active() && !event::poll(Duration::from_millis(30))? {
            continue;
        }

        let event = event::read()?;
        // Non-key events (resize, focus, mouse) should not preserve the stale
        // "No tiles moved" message from a previous invalid directional input.
        let Event::Key(key) = event else {
            state.clear_invalid_move();
            continue;
        };

        let mov = match (key.code, key.modifiers) {
            (KeyCode::Char('q'), _)
            | (KeyCode::Char('Q'), _)
            | (KeyCode::Char('c'), KeyModifiers::CONTROL)
            | (KeyCode::Esc, _) => break,
            (KeyCode::Char('a'), _) | (KeyCode::Char('A'), _) | (KeyCode::Left, _) => Move::Left,
            (KeyCode::Char('d'), _) | (KeyCode::Char('D'), _) | (KeyCode::Right, _) => Move::Right,
            (KeyCode::Char('w'), _) | (KeyCode::Char('W'), _) | (KeyCode::Up, _) => Move::Up,
            (KeyCode::Char('s'), _) | (KeyCode::Char('S'), _) | (KeyCode::Down, _) => Move::Down,
            _ => Move::Dont,
        };

        let end_msg = match state.apply_move(mov) {
            Status::On => continue,
            Status::Won => "You won!  Press any key to exit.",
            Status::Lost => "Game over!  Press any key to exit.",
        };

        state.render_end_message(end_msg)?;
        event::read()?;
        break;
    }

    Ok(())
}
