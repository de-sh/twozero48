use super::Tile;

/// The maximum size of the board (square root of usize::MAX)
///
/// i.e. `u32::MAX` for 64bit and `u16::MAX` for 32bit architectures
const MAX_SIZE: usize = usize::MAX.isqrt();

/// Ensures the board size is valid
fn check_size(size: usize) {
    assert!(
        (2..=MAX_SIZE).contains(&size),
        "board size must be between 2 and {MAX_SIZE}"
    );
}

/// A Square (size x size) 2D grid of tiles for the 2048 game
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    tiles: Vec<Tile>,
    size: usize,
}

impl Board {
    pub fn new(size: usize) -> Self {
        check_size(size);
        Self {
            tiles: vec![Tile::EMPTY; size * size],
            size,
        }
    }

    #[cfg(test)]
    pub fn from_grid<const N: usize>(grid: [[Tile; N]; N]) -> Self {
        check_size(N);
        let tiles = (0..N)
            .flat_map(|row| (0..N).map(move |col| grid[row][col]))
            .collect();
        Self { tiles, size: N }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn move_left(&mut self) -> usize {
        let mut score = 0;
        for row in 0..self.size {
            score += self.compress_lane(row * self.size, 1);
        }
        score
    }

    pub fn move_right(&mut self) -> usize {
        let mut score = 0;
        for row in 0..self.size {
            score += self.compress_lane(row * self.size + self.size - 1, -1);
        }
        score
    }

    pub fn move_up(&mut self) -> usize {
        let mut score = 0;
        for col in 0..self.size {
            score += self.compress_lane(col, self.size as isize);
        }
        score
    }

    pub fn move_down(&mut self) -> usize {
        let mut score = 0;
        for col in 0..self.size {
            score += self.compress_lane((self.size - 1) * self.size + col, -(self.size as isize));
        }
        score
    }

    /// Compresses one row or column in lane order.
    fn compress_lane(&mut self, start: usize, step: isize) -> usize {
        let mut write = 0;
        // A pending tile to be merged with the next tile if equal.
        let mut pending: Option<Tile> = None;
        let mut score = 0;

        for read in 0..self.size {
            let tile = self.tiles[self.lane_index(start, step, read)];

            if tile == Tile::EMPTY {
                continue;
            }

            match pending {
                None => pending = Some(tile),
                Some(prev) if prev == tile => {
                    let promoted = prev.promote();
                    let write_idx = self.lane_index(start, step, write);
                    self.tiles[write_idx] = promoted;
                    write += 1;
                    score += promoted.score();
                    pending = None;
                }
                Some(prev) => {
                    let write_idx = self.lane_index(start, step, write);
                    self.tiles[write_idx] = prev;
                    write += 1;
                    pending = Some(tile);
                }
            }
        }

        if let Some(tile) = pending {
            let write_idx = self.lane_index(start, step, write);
            self.tiles[write_idx] = tile;
            write += 1;
        }

        for i in write..self.size {
            let idx = self.lane_index(start, step, i);
            self.tiles[idx] = Tile::EMPTY;
        }

        score
    }

    fn offset(&self, row: usize, col: usize) -> usize {
        assert!(
            row < self.size && col < self.size,
            "board index out of bounds"
        );
        row * self.size + col
    }

    fn lane_index(&self, start: usize, step: isize, offset: usize) -> usize {
        let idx = start as isize + step * offset as isize;
        debug_assert!(idx >= 0);
        debug_assert!((idx as usize) < self.tiles.len());
        idx as usize
    }

    pub fn max_tile(&self) -> Tile {
        self.tiles.iter().max().copied().unwrap_or(Tile::EMPTY)
    }

    pub fn contains(&self, tile: Tile) -> bool {
        self.tiles.iter().any(|t| t.exp == tile.exp)
    }

    /// Verify if board is filled and no valid moves left
    pub fn is_locked(&self) -> bool {
        for row in 0..self.size {
            for col in 0..self.size {
                let idx = row * self.size + col;
                let tile = self.tiles[idx];

                // Check if tile is empty
                if tile == Tile::EMPTY {
                    return false;
                }

                // Check right neighbor
                if col < self.size - 1 && tile == self.tiles[idx + 1] {
                    return false;
                }

                // Check down neighbor
                if row < self.size - 1 && tile == self.tiles[idx + self.size] {
                    return false;
                }
            }
        }

        true
    }
}

impl std::ops::Index<(usize, usize)> for Board {
    type Output = Tile;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        let offset = self.offset(row, col);
        &self.tiles[offset]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        let offset = self.offset(row, col);
        &mut self.tiles[offset]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_compresses_contiguous_rows_without_scratch_space() {
        let mut board = Board::from_grid([
            [Tile::TWO, Tile::EMPTY, Tile::TWO, Tile::FOUR],
            [Tile::FOUR, Tile::FOUR, Tile::FOUR, Tile::FOUR],
            [Tile::TWO, Tile::TWO, Tile::TWO, Tile::EMPTY],
            [Tile::EMPTY, Tile::EMPTY, Tile::EMPTY, Tile::EMPTY],
        ]);
        let tiles_ptr = board.tiles.as_ptr();

        let score = board.move_left();

        assert_eq!(board.tiles.as_ptr(), tiles_ptr);
        assert_eq!(score, 4 + 8 + 8 + 4);
        assert_eq!(
            board,
            Board::from_grid([
                [Tile::FOUR, Tile::FOUR, Tile::EMPTY, Tile::EMPTY],
                [Tile::new(3), Tile::new(3), Tile::EMPTY, Tile::EMPTY],
                [Tile::FOUR, Tile::TWO, Tile::EMPTY, Tile::EMPTY],
                [Tile::EMPTY, Tile::EMPTY, Tile::EMPTY, Tile::EMPTY],
            ]),
        );
    }

    #[test]
    fn right_uses_the_same_lane_compressor_in_reverse_order() {
        let mut board = Board::from_grid([
            [Tile::TWO, Tile::EMPTY, Tile::TWO, Tile::FOUR],
            [Tile::TWO, Tile::TWO, Tile::TWO, Tile::EMPTY],
            [Tile::EMPTY, Tile::EMPTY, Tile::EMPTY, Tile::EMPTY],
            [Tile::FOUR, Tile::FOUR, Tile::FOUR, Tile::FOUR],
        ]);

        let score = board.move_right();

        assert_eq!(score, 4 + 4 + 8 + 8);
        assert_eq!(
            board,
            Board::from_grid([
                [Tile::EMPTY, Tile::EMPTY, Tile::FOUR, Tile::FOUR],
                [Tile::EMPTY, Tile::EMPTY, Tile::TWO, Tile::FOUR],
                [Tile::EMPTY, Tile::EMPTY, Tile::EMPTY, Tile::EMPTY],
                [Tile::EMPTY, Tile::EMPTY, Tile::new(3), Tile::new(3)],
            ]),
        );
    }

    #[test]
    fn up_compresses_strided_columns_on_row_major_storage() {
        let mut board = Board::from_grid([
            [Tile::TWO, Tile::FOUR, Tile::EMPTY, Tile::TWO],
            [Tile::EMPTY, Tile::FOUR, Tile::EMPTY, Tile::TWO],
            [Tile::TWO, Tile::FOUR, Tile::EMPTY, Tile::TWO],
            [Tile::FOUR, Tile::FOUR, Tile::EMPTY, Tile::EMPTY],
        ]);

        let score = board.move_up();

        assert_eq!(score, 4 + 8 + 8 + 4);
        assert_eq!(
            board,
            Board::from_grid([
                [Tile::FOUR, Tile::new(3), Tile::EMPTY, Tile::FOUR],
                [Tile::FOUR, Tile::new(3), Tile::EMPTY, Tile::TWO],
                [Tile::EMPTY, Tile::EMPTY, Tile::EMPTY, Tile::EMPTY],
                [Tile::EMPTY, Tile::EMPTY, Tile::EMPTY, Tile::EMPTY],
            ]),
        );
    }

    #[test]
    fn down_compresses_strided_columns_in_reverse_order() {
        let mut board = Board::from_grid([
            [Tile::TWO, Tile::FOUR, Tile::EMPTY, Tile::TWO],
            [Tile::EMPTY, Tile::FOUR, Tile::EMPTY, Tile::TWO],
            [Tile::TWO, Tile::FOUR, Tile::EMPTY, Tile::TWO],
            [Tile::FOUR, Tile::FOUR, Tile::EMPTY, Tile::EMPTY],
        ]);

        let score = board.move_down();

        assert_eq!(score, 4 + 8 + 8 + 4);
        assert_eq!(
            board,
            Board::from_grid([
                [Tile::EMPTY, Tile::EMPTY, Tile::EMPTY, Tile::EMPTY],
                [Tile::EMPTY, Tile::EMPTY, Tile::EMPTY, Tile::EMPTY],
                [Tile::FOUR, Tile::new(3), Tile::EMPTY, Tile::TWO],
                [Tile::FOUR, Tile::new(3), Tile::EMPTY, Tile::FOUR],
            ]),
        );
    }

    #[test]
    fn indexing_maps_rows_and_columns_to_flat_row_major_offsets() {
        let mut board = Board::new(4);

        board[(2, 1)] = Tile::new(5);
        board[(3, 3)] = Tile::new(6);

        assert_eq!(board.tiles[9], Tile::new(5));
        assert_eq!(board.tiles[15], Tile::new(6));
        assert_eq!(board[(2, 1)], Tile::new(5));
        assert_eq!(board[(3, 3)], Tile::new(6));
    }
}
