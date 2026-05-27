use twozero48::{Board, Tile};

pub fn sample_board() -> Board {
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
