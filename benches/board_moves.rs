use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use twozero48::Move;

mod common;

fn board_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("board_moves");

    for mov in [Move::Left, Move::Right, Move::Up, Move::Down] {
        group.bench_function(BenchmarkId::new("4x4", format!("{mov:?}")), |b| {
            let seed = common::sample_board();
            b.iter(|| {
                let mut board = seed.clone();
                black_box(match mov {
                    Move::Left => board.move_left(),
                    Move::Right => board.move_right(),
                    Move::Up => board.move_up(),
                    Move::Down => board.move_down(),
                });
                black_box(board);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, board_moves);
criterion_main!(benches);
