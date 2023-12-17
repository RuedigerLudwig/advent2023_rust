use super::{DayTrait, DayType, RResult};
use crate::common::{
    direction::Direction,
    path_finder::{find_best_path, ItemSkipper, PathFinder},
    pos2::Pos2,
};
use itertools::Itertools;
use std::{
    collections::{BinaryHeap, HashSet},
    marker::PhantomData,
    num,
    str::FromStr,
};

const DAY_NUMBER: DayType = 17;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let map: HeatMap<HeatSkipper> = input.parse()?;
        Ok(map.best_path()?.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let map: HeatMap<UltraHeatSkipper> = input.parse()?;
        Ok(map.best_path()?.into())
    }
}

struct HeatMap<S: ItemSkipper> {
    map: Vec<Vec<u32>>,
    s: PhantomData<S>,
}

impl<S: ItemSkipper<Item = HeatFlow>> HeatMap<S> {
    fn print(&self, _heat_flow: &HeatFlow) {
        /*
           for y in 0..self.map.len() {
               for x in 0..self.map[0].len() {
                   if let Some(_) = heat_flow.seen.get(&Pos2::new(x, y)) {
                       print!("{}", format!("{}", self.map[y][x]).red())
                   } else {
                       print!("{}", format!("{}", self.map[y][x]).blue())
                   }
               }
               println!();
           }
           println!(
               "{}",
               heat_flow
                   .progress
                   .iter()
                   .map(|(p1, p2)| format!("{} ({})", p1, p2))
                   .join(",")
           );
        */
    }

    pub fn best_path(self) -> Result<u32, DayError> {
        find_best_path(self)
            .map(|heat_flow| heat_flow.loss)
            .ok_or(DayError::NoBestPathFound)
    }
}

struct HeatFlow {
    loss: u32,
    straight: usize,
    prev_straight: usize,
    pos: Pos2<usize>,
    direction: Direction,
    //seen: HashMap<Pos2<usize>, Direction>,
    //progress: Vec<(u32, usize)>,
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

struct HeatSkipper {
    visited: HashSet<(Pos2<usize>, Direction, usize, usize)>,
}

impl ItemSkipper for HeatSkipper {
    type Item = HeatFlow;

    fn init() -> Self {
        HeatSkipper {
            visited: HashSet::new(),
        }
    }

    fn skip_item(&mut self, item: &Self::Item) -> bool {
        let skip = item.straight > 3
            || self.visited.contains(&(
                item.pos,
                item.direction,
                item.straight,
                item.prev_straight,
            ));
        self.visited
            .insert((item.pos, item.direction, item.straight, item.prev_straight));
        skip
    }
}

struct UltraHeatSkipper {
    visited: HashSet<(Pos2<usize>, usize, usize)>,
}

impl ItemSkipper for UltraHeatSkipper {
    type Item = HeatFlow;

    fn init() -> Self {
        Self {
            visited: HashSet::new(),
        }
    }

    fn skip_item(&mut self, item: &Self::Item) -> bool {
        if (1..=3).contains(&item.prev_straight)
            || item.straight > 10
            || self
                .visited
                .contains(&(item.pos, item.straight, item.prev_straight))
        {
            true
        } else {
            self.visited
                .insert((item.pos, item.straight, item.prev_straight));
            false
        }
    }

    fn skip_when_finished(&self, item: &Self::Item) -> bool {
        !(4..=10).contains(&item.straight) || !(4..=10).contains(&item.prev_straight)
    }
}

impl<S: ItemSkipper<Item = HeatFlow>> PathFinder for HeatMap<S> {
    type Item = HeatFlow;
    type Queue = BinaryHeap<Self::Item>;
    type Skipper = S;

    fn get_start_item(&self) -> Self::Item {
        HeatFlow {
            loss: 0,
            straight: 0,
            prev_straight: 0,
            pos: Pos2::new(0, 0),
            direction: Direction::West,
            //seen: HashMap::new(),
            //progress: vec![],
        }
    }

    fn is_finished(&self, item: &Self::Item) -> bool {
        let maybe_finished =
            item.pos.x() == self.map[0].len() - 1 && item.pos.y() == self.map.len() - 1;
        if maybe_finished {
            self.print(item);
        }
        maybe_finished
    }

    fn get_next_states<'a>(
        &'a self,
        item: &'a Self::Item,
    ) -> impl Iterator<Item = Self::Item> + 'a {
        Direction::iter().filter_map(|direction| {
            let mut straight = 1;
            let mut prev_straight = item.prev_straight;
            if item.straight != 0 {
                if direction == item.direction.turn_back() {
                    return None;
                }
                if direction == item.direction {
                    straight = item.straight + 1;
                } else {
                    prev_straight = item.straight;
                }
            }
            item.pos
                .safe_matrix_add_and_get(&self.map, direction)
                .map(|(pos, loss)| {
                    /*
                    let mut seen = item.seen.clone();
                    seen.insert(pos, direction);
                    let mut progress = item.progress.clone();
                    progress.push((item.loss + *loss, straight));
                     */

                    HeatFlow {
                        loss: item.loss + *loss,
                        straight,
                        prev_straight,
                        pos,
                        direction,
                        //seen,
                        //progress,
                    }
                })
        })
    }
}

impl<S: ItemSkipper> FromStr for HeatMap<S> {
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
            s: PhantomData,
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
        //1073
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(102);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        //1244 too low
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
