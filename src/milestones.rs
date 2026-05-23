use twozero48::Tile;

const MILESTONES: [Tile; 4] = [Tile::new(7), Tile::new(9), Tile::new(10), Tile::new(12)];

/// Milestones are to be celebrated, but they also need to be tracked!
/// We have selected 128, 512, 1024 and 4096 as milestone tiles.
pub struct MilestoneChecker {
    previous_largest: Tile,
}

impl MilestoneChecker {
    pub fn new(previous_largest: Tile) -> Self {
        Self { previous_largest }
    }

    pub fn is_milestone(&mut self, current_largest: Tile) -> bool {
        let result =
            MILESTONES.iter().rev().copied().find(|&milestone| {
                self.previous_largest < milestone && milestone <= current_largest
            });
        self.previous_largest = current_largest;
        result.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn before_first_milestone() {
        let mut checker = MilestoneChecker::new(Tile::EMPTY);
        assert!(!checker.is_milestone(Tile::new(6)));
    }

    #[test]
    fn first_milestone() {
        let mut checker = MilestoneChecker::new(Tile::new(6));
        assert!(checker.is_milestone(Tile::new(7)));
    }

    #[test]
    fn milestone_was_already_reached() {
        let mut checker = MilestoneChecker::new(Tile::new(7));
        assert!(!checker.is_milestone(Tile::new(7)));
    }

    #[test]
    fn next_milestone() {
        let mut checker = MilestoneChecker::new(Tile::new(7));
        assert!(checker.is_milestone(Tile::new(9)));
    }
}
