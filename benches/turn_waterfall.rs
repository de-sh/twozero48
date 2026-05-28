use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::{SeedableRng, rngs::StdRng};
use ratatui::backend::TestBackend;
use twozero48::{Board, Game, Move, State, Tile, Tui};

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
                || utils_for(seed.clone()),
                |(mut state, mut tui)| {
                    let status = state.apply_move(mov);
                    black_box(status);

                    for _ in 0..ANIMATION_FRAMES {
                        state.tick_effects();
                        black_box(state.effects_active());
                        let screen = state.as_screen();
                        tui.draw(screen).expect("rendering to TestBackend failed");
                    }

                    black_box(state);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn utils_for(board: Board) -> (State<StdRng>, Tui<TestBackend>) {
    (
        State::new(Game::from_board_with_rng(
            board,
            Tile::new(16),
            StdRng::seed_from_u64(0),
        )),
        Tui::from_backend(TestBackend::new(WIDTH, HEIGHT))
            .expect("constructing TestBackend terminal failed"),
    )
}

criterion_group!(benches, turn_waterfall);
criterion_main!(benches);
