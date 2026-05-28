

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::{cell::RefCell, collections::VecDeque, io, rc::Rc, error::Error};

    use ratatui::Terminal;
    use ratzilla::{
        DomBackend, WebRenderer,
        event::{KeyCode, KeyEvent},
    };
    use twozero48::{Game, Input, Move, State, Tile};

    const BOARD_SIZE: usize = 4;
    /// 2^11, i.e. 2048
    const WINNING_TILE: Tile = Tile::new(11);
    const TERMINAL_ID: &str = "twozero48-terminal";

    fn from_key(event: KeyEvent) -> Input {
        match event.code {
            KeyCode::Char('c') if event.ctrl => Input::Quit,
            KeyCode::Esc => Input::Quit,
            KeyCode::Char(ch) => Input::from_char(ch),
            KeyCode::Left => Input::Move(Move::Left),
            KeyCode::Right => Input::Move(Move::Right),
            KeyCode::Up => Input::Move(Move::Up),
            KeyCode::Down => Input::Move(Move::Down),
            _ => Input::Ignore,
        }
    }

    type InputQueue = Rc<RefCell<VecDeque<Input>>>;

    fn close_window() {
        if let Some(window) = web_sys::window() {
            let _ = window.close();
        }
    }

    pub fn run()-> Result<(), Box<dyn Error>> {
        console_error_panic_hook::set_once();

        let backend = DomBackend::new_by_id(TERMINAL_ID)?;
        let terminal = Terminal::new(backend)?;
        let inputs: InputQueue = Rc::new(RefCell::new(VecDeque::new()));

        terminal.on_key_event({
            let inputs = Rc::clone(&inputs);
            move |event| {
                inputs.borrow_mut().push_back(from_key(event));
            }
        });

        let mut state = Game::new(BOARD_SIZE, WINNING_TILE)
            .map(State::new)
            .map_err(io::Error::other)?;
        terminal.draw_web(move |frame| {
            loop {
                let input = inputs.borrow_mut().pop_front();
                match input {
                    Some(Input::Quit) => {
                        close_window();
                        break;
                    }
                    None => break,
                    Some(input) => state.handle_input(input),
                };
            }

            state.tick_effects();
            state.as_screen().render(frame);
        });

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(target_arch = "wasm32"))]
    unimplemented!("Supposed to only run on wasm targets");

    #[cfg(target_arch = "wasm32")]
    wasm::run()
}
