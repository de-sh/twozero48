use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use twozero48::{Board, Tile};

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

    fn apply(self, board: &mut Board) -> usize {
        match self {
            Self::Left => board.move_left(),
            Self::Right => board.move_right(),
            Self::Up => board.move_up(),
            Self::Down => board.move_down(),
        }
    }
}

fn board_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("board_moves");

    for direction in [
        Direction::Left,
        Direction::Right,
        Direction::Up,
        Direction::Down,
    ] {
        group.bench_function(BenchmarkId::new("4x4", direction.name()), |b| {
            let seed = sample_board();
            b.iter(|| {
                let mut board = seed.clone();
                black_box(direction.apply(&mut board));
                black_box(board);
            });
        });
    }

    group.finish();
}

fn sample_board() -> Board {
    let mut board = Board::new(4);
    let grid = [
        [Tile::TWO, Tile::EMPTY, Tile::TWO, Tile::FOUR],
        [Tile::FOUR, Tile::FOUR, Tile::EMPTY, Tile::TWO],
        [Tile::EMPTY, Tile::TWO, Tile::new(3), Tile::new(3)],
        [Tile::new(4), Tile::EMPTY, Tile::EMPTY, Tile::new(4)],
    ];

    for (row, tiles) in grid.into_iter().enumerate() {
        for (col, tile) in tiles.into_iter().enumerate() {
            board[(row, col)] = tile;
        }
    }

    board
}

criterion_group!(benches, board_moves);
criterion_main!(benches);
