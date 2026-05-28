use std::{error::Error, time::Duration};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use twozero48::{Game, Input, Move, State, Tile, Tui};

fn from_key(key: KeyEvent) -> Input {
    match (key.code, key.modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => Input::Quit,
        (KeyCode::Char(ch), _) => Input::from_char(ch),
        (KeyCode::Left, _) => Input::Move(Move::Left),
        (KeyCode::Right, _) => Input::Move(Move::Right),
        (KeyCode::Up, _) => Input::Move(Move::Up),
        (KeyCode::Down, _) => Input::Move(Move::Down),
        _ => Input::Ignore,
    }
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
    let opts = Opts::parse();
    let mut state = State::new(Game::new(opts.board_size, opts.winning)?);
    let mut tui = Tui::new()?;

    loop {
        state.tick_effects();
        tui.draw(state.as_screen())?;

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

        match from_key(key) {
            Input::Quit => break,
            input => state.handle_input(input),
        }
    }

    Ok(())
}
