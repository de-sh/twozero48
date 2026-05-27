use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use ratatui::backend::TestBackend;
use twozero48::{Board, Move, Play, State, Status, Tile, Tui};

const ANIMATION_FRAMES: usize = 4;
const WIDTH: u16 = 80;
const HEIGHT: u16 = 24;

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Up => "up",
            Self::Down => "down",
        }
    }

    fn mov(self) -> Move {
        match self {
            Self::Left => Move::Left,
            Self::Right => Move::Right,
            Self::Up => Move::Up,
            Self::Down => Move::Down,
        }
    }
}

fn turn_waterfall(c: &mut Criterion) {
    let mut group = c.benchmark_group("turn_waterfall");

    for direction in [
        Direction::Left,
        Direction::Right,
        Direction::Up,
        Direction::Down,
    ] {
        group.bench_function(BenchmarkId::new("4x4", direction.name()), |b| {
            let seed = sample_board();
            let mov = direction.mov();

            b.iter_batched(
                || state_for(seed.clone()),
                |mut state| {
                    let status = state.apply_move(mov);
                    black_box(status);

                    for _ in 0..ANIMATION_FRAMES {
                        state.tick_effects();
                        black_box(state.effects_active());
                        state
                            .render_board()
                            .expect("rendering to TestBackend failed");
                    }

                    black_box(state);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// A deterministic mock game for benchmarking.
struct MockGame {
    board: Board,
    winning: Tile,
    score: usize,
    spawn_cursor: usize,
    spawn_count: usize,
}

impl MockGame {
    fn from_board(board: Board, winning: Tile) -> Self {
        Self {
            board,
            winning,
            score: 0,
            spawn_cursor: 0,
            spawn_count: 0,
        }
    }
}

impl Play for MockGame {
    fn board(&self) -> &Board {
        &self.board
    }

    fn winning(&self) -> Tile {
        self.winning
    }

    fn score(&self) -> usize {
        self.score
    }

    fn largest_tile(&self) -> Tile {
        self.board.max_tile()
    }

    fn status(&self) -> Status {
        if self.board.contains(self.winning) {
            Status::Won
        } else if self.board.is_locked() {
            Status::Lost
        } else {
            Status::On
        }
    }

    fn mover(&mut self, mov: Move) -> bool {
        let before = self.board.clone();
        self.score += match mov {
            Move::Left => self.board.move_left(),
            Move::Right => self.board.move_right(),
            Move::Up => self.board.move_up(),
            Move::Down => self.board.move_down(),
        };
        self.board != before
    }

    /// spawns a tile in deterministic order.
    fn spawn(&mut self) {
        let len = self.board.size() * self.board.size();
        for _ in 0..len {
            let idx = self.spawn_cursor % len;
            self.spawn_cursor += 1;
            let cell = (idx / self.board.size(), idx % self.board.size());
            if self.board[cell] == Tile::EMPTY {
                self.board[cell] = match self.spawn_count % 3 {
                    2 => Tile::FOUR,
                    _ => Tile::TWO,
                };
                self.spawn_count += 1;
                return;
            }
        }
    }
}

fn state_for(board: Board) -> State<MockGame, TestBackend> {
    let tui = Tui::from_backend(TestBackend::new(WIDTH, HEIGHT))
        .expect("constructing TestBackend terminal failed");
    State::with_tui(MockGame::from_board(board, Tile::new(16)), tui)
}

fn sample_board() -> Board {
    let mut board = Board::new(4).expect("board created");
    let grid = [
        [Tile::TWO, Tile::TWO, Tile::FOUR, Tile::FOUR],
        [Tile::new(3), Tile::new(4), Tile::new(5), Tile::new(6)],
        [Tile::new(3), Tile::new(7), Tile::new(8), Tile::new(9)],
        [Tile::new(10), Tile::new(11), Tile::new(12), Tile::new(13)],
    ];

    for (row, tiles) in grid.into_iter().enumerate() {
        for (col, tile) in tiles.into_iter().enumerate() {
            board[(row, col)] = tile;
        }
    }

    board
}

criterion_group!(benches, turn_waterfall);
criterion_main!(benches);
