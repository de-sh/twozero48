use twozero48::Tile;

const MILESTONES: [Tile; 4] = [
    Tile::OneHundredTwentyEight,
    Tile::FiveHundredTwelve,
    Tile::OneThousandTwentyFour,
    Tile::FourHundredNinetySix,
];

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
        let mut checker = MilestoneChecker::new(Tile::Empty);
        assert!(!checker.is_milestone(Tile::SixtyFour));
    }

    #[test]
    fn first_milestone() {
        let mut checker = MilestoneChecker::new(Tile::SixtyFour);
        assert!(checker.is_milestone(Tile::OneHundredTwentyEight));
    }

    #[test]
    fn milestone_was_already_reached() {
        let mut checker = MilestoneChecker::new(Tile::OneHundredTwentyEight);
        assert!(!checker.is_milestone(Tile::OneHundredTwentyEight));
    }

    #[test]
    fn next_milestone() {
        let mut checker = MilestoneChecker::new(Tile::OneHundredTwentyEight);
        assert!(checker.is_milestone(Tile::FiveHundredTwelve));
    }
}
