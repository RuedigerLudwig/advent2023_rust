use self::debug::HeatDebugger;
use super::{DayTrait, DayType, RResult};
use crate::common::{
    direction::Direction,
    path_finder::{find_best_path, FingerprintItem, FingerprintSkipper, PathFinder},
    pos2::Pos2,
};
use itertools::Itertools;
use std::{collections::BinaryHeap, num, str::FromStr};

#[cfg(feature = "debug")]
use colored::Colorize;

const DAY_NUMBER: DayType = 17;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let map: HeatMap = input.parse()?;
        Ok(map.best_path()?.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let mut map: HeatMap = input.parse()?;
        map.set_checker(HeatChecker::new(4, 10));
        Ok(map.best_path()?.into())
    }
}

#[derive(Debug, Clone, Copy)]
struct HeatChecker {
    min_steps: usize,
    max_steps: usize,
}

impl HeatChecker {
    pub fn new(min_steps: usize, max_steps: usize) -> Self {
        assert!(max_steps >= min_steps);
        Self {
            min_steps,
            max_steps,
        }
    }

    pub fn check(&self, straight: usize) -> bool {
        (self.min_steps..=self.max_steps).contains(&straight)
    }
}

#[cfg(feature = "debug")]
mod debug {
    use super::*;
    use crate::common::{direction::Direction, pos2::Pos2};
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    pub struct HeatDebugger {
        seen: HashMap<Pos2<usize>, Direction>,
        progress: Vec<(u32, usize)>,
    }

    impl HeatDebugger {
        pub fn new() -> Self {
            Self {
                seen: HashMap::new(),
                progress: vec![],
            }
        }

        pub fn print(&self, heat_map: &HeatMap) {
            for y in 0..heat_map.map.len() {
                for x in 0..heat_map.map[0].len() {
                    if self.seen.get(&Pos2::new(x, y)).is_some() {
                        print!("{}", format!("{}", heat_map.map[y][x]).red())
                    } else {
                        print!("{}", format!("{}", heat_map.map[y][x]).blue())
                    }
                }
                println!();
            }
            println!(
                "{}",
                self.progress
                    .iter()
                    .map(|(p1, p2)| format!("{} ({})", p1, p2))
                    .join(", ")
            )
        }

        pub fn push(&mut self, pos: Pos2<usize>, direction: Direction, loss: u32, straight: usize) {
            self.progress.push((loss, straight));
            self.seen.insert(pos, direction);
        }
    }
}
#[cfg(not(feature = "debug"))]
mod debug {
    use super::*;
    #[derive(Debug, Clone)]
    pub struct HeatDebugger;
    impl HeatDebugger {
        #[inline]
        pub fn new() -> HeatDebugger {
            HeatDebugger
        }

        #[inline]
        pub fn print(&self, _heat_map: &HeatMap) {}

        #[inline]
        pub fn push(
            &mut self,
            _pos: Pos2<usize>,
            _direction: Direction,
            _loss: u32,
            _straight: usize,
        ) {
        }
    }
}

struct HeatFlow {
    loss: u32,
    straight: usize,
    pos: Pos2<usize>,
    direction: Option<Direction>,
    debugger: HeatDebugger,
}

impl Eq for HeatFlow {}

impl PartialEq for HeatFlow {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl PartialOrd for HeatFlow {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeatFlow {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match other.loss.cmp(&self.loss) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.pos.abs().cmp(&other.pos.abs())
    }
}

impl FingerprintItem for HeatFlow {
    type Fingerprint = (Pos2<usize>, Option<Direction>, usize);

    fn get_fingerprint(&self) -> Self::Fingerprint {
        (self.pos, self.direction, self.straight)
    }
}

struct HeatMap {
    map: Vec<Vec<u32>>,
    checker: HeatChecker,
}

impl HeatMap {
    pub fn set_checker(&mut self, checker: HeatChecker) {
        self.checker = checker;
    }

    pub fn best_path(self) -> Result<u32, DayError> {
        find_best_path(self)
            .map(|heat_flow| heat_flow.loss)
            .ok_or(DayError::NoBestPathFound)
    }
}

impl PathFinder for HeatMap {
    type Item = HeatFlow;
    type Queue = BinaryHeap<Self::Item>;
    type Skipper = FingerprintSkipper<HeatFlow>;

    fn get_start_item(&self) -> Self::Item {
        HeatFlow {
            loss: 0,
            straight: 0,
            pos: Pos2::new(0, 0),
            direction: None,
            debugger: HeatDebugger::new(),
        }
    }

    fn is_finished(&self, item: &Self::Item) -> bool {
        let maybe_finished =
            item.pos.x() == self.map[0].len() - 1 && item.pos.y() == self.map.len() - 1;
        if maybe_finished {
            item.debugger.print(self);
        }
        maybe_finished
    }

    fn get_next_states<'a>(
        &'a self,
        item: &'a Self::Item,
    ) -> impl Iterator<Item = Self::Item> + 'a {
        Direction::iter().filter_map(|direction| {
            let mut straight = item.straight;
            let mut steps = self.checker.min_steps;
            if let Some(prev_direction) = item.direction {
                if direction == prev_direction.turn_back() {
                    return None;
                }
                if direction == prev_direction {
                    steps = 1;
                } else {
                    straight = 0;
                }
            }
            let mut loss = item.loss;
            let mut pos = item.pos;
            let mut debugger = item.debugger.clone();
            for _ in 0..steps {
                let (next_pos, &next_loss) = pos.safe_matrix_add_and_get(&self.map, direction)?;
                straight += 1;
                loss += next_loss;
                pos = next_pos;
                debugger.push(pos, direction, loss, straight);
            }
            if !self.checker.check(straight) {
                return None;
            }

            Some(HeatFlow {
                loss,
                straight,
                pos,
                direction: Some(direction),
                debugger,
            })
        })
    }
}

impl FromStr for HeatMap {
    type Err = DayError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let map: Vec<Vec<u32>> = input
            .lines()
            .map(|row| {
                row.chars()
                    .map(|c| c.to_digit(10).ok_or(DayError::NoAsciiNumber(c)))
                    .try_collect()
            })
            .try_collect()?;
        if map.is_empty() || map[0].is_empty() {
            return Err(DayError::HeadMapMustNotBeEmpty);
        }
        if !map.iter().map(|row| row.len()).all_equal() {
            return Err(DayError::HeatMapMustBeRectangle);
        }
        Ok(Self {
            map,
            checker: HeatChecker::new(1, 3),
        })
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Not an Ascii Digit: {0}")]
    NoAsciiNumber(char),
    #[error("heat Map must not be empty")]
    HeadMapMustNotBeEmpty,
    #[error("Heat Map must be a reactangle")]
    HeatMapMustBeRectangle,
    #[error("no best path found")]
    NoBestPathFound,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::days::{read_string, ResultType, UnitResult};

    #[test]
    fn test_part1() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(102);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(94);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn example2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;
        let expected = ResultType::Integer(71);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }
}
