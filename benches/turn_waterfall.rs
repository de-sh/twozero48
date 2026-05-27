use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::{SeedableRng, rngs::StdRng};
use ratatui::backend::TestBackend;
use twozero48::{Board, Game, Move, State, Tile};

mod common;

const ANIMATION_FRAMES: usize = 4;
const WIDTH: u16 = 80;
const HEIGHT: u16 = 24;

fn turn_waterfall(c: &mut Criterion) {
    let mut group = c.benchmark_group("turn_waterfall");

    for mov in [Move::Left, Move::Right, Move::Up, Move::Down] {
        group.bench_function(BenchmarkId::new("4x4", format!("{mov:?}")), |b| {
            let seed = common::sample_board();

            b.iter_batched(
                || state_for(seed.clone()),
                |mut state| {
                    let status = state.apply_move(mov);
                    black_box(status);

                    for _ in 0..ANIMATION_FRAMES {
                        state.tick_effects();
                        black_box(state.effects_active());
                        state.render_tui().expect("rendering to TestBackend failed");
                    }

                    black_box(state);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn state_for(board: Board) -> State<TestBackend, StdRng> {
    State::with_backend(
        Game::from_board_with_rng(board, Tile::new(16), StdRng::seed_from_u64(0)),
        TestBackend::new(WIDTH, HEIGHT),
    )
    .expect("constructing TestBackend terminal failed")
}

criterion_group!(benches, turn_waterfall);
criterion_main!(benches);
