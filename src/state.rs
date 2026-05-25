use std::io;

use twozero48::{Game, Move, Status, Tile};

use crate::tui::{MoveEffects, Tui};

pub struct State {
    terminal: Tui,
    game: Game,
    move_effects: MoveEffects,
    valid_move: bool,
    last_milestone: Option<Tile>,
    previous_largest: Tile,
}

impl State {
    pub fn new(game: Game) -> io::Result<Self> {
        Ok(Self {
            terminal: Tui::new()?,
            game,
            move_effects: MoveEffects::new(),
            valid_move: true,
            last_milestone: None,
            previous_largest: Tile::EMPTY,
        })
    }

    pub fn tick_effects(&mut self) {
        self.move_effects.tick();
    }

    pub fn effects_active(&self) -> bool {
        self.move_effects.is_active()
    }

    pub fn clear_invalid_move(&mut self) {
        self.valid_move = true;
    }

    pub fn apply_move(&mut self, mov: Move) -> Status {
        if matches!(mov, Move::Dont) {
            self.valid_move = false;
            return self.game.status();
        }

        let old_board = self.game.board().clone();

        self.valid_move = self.game.mover(mov);

        if self.valid_move {
            self.move_effects
                .record_move(mov, &old_board, self.game.board());
            self.game.refresh();

            let current_largest = self.game.largest_tile();
            self.check_milestone(current_largest);
        }

        self.game.status()
    }

    pub fn render_board(&mut self) -> io::Result<()> {
        let (x_shift, y_shift) = self.move_effects.shift();
        let message = if self.valid_move {
            None
        } else {
            Some("No tiles moved — try a different direction")
        };

        self.terminal.render_board(
            &self.game,
            message,
            x_shift,
            y_shift,
            self.move_effects.flash(),
        )
    }

    pub fn render_end_message(&mut self, message: &str) -> io::Result<()> {
        self.move_effects.clear();
        self.terminal
            .render_board(&self.game, Some(message), 0, 0, self.move_effects.flash())
    }

    fn check_milestone(&mut self, tile: Tile) {
        if tile.exponent() <= self.previous_largest.exponent() {
            return;
        }
        self.previous_largest = tile;
        if tile > Tile::ONE_TWO_EIGHT {
            self.last_milestone = Some(tile);
        }
    }
}
