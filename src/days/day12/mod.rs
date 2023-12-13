use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{collections::HashMap, num, ops::Add, str::FromStr};

const DAY_NUMBER: DayType = 12;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let result = input
            .lines()
            .map(|line| line.parse::<SpringList>())
            .map_ok(|sl| sl.get_arrangements())
            .fold_ok(0, Add::add)?;
        Ok(result.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let result = input
            .lines()
            .map(|line| line.parse::<SpringList>())
            .map_ok(|sl| sl.get_long_arrangements())
            .fold_ok(0, Add::add)?;
        Ok(result.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Unknown Icon: {0}")]
    UnknownSpring(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Icon {
    Unknown,
    Operational,
    Damaged,
}

impl From<&Icon> for char {
    #[inline]
    fn from(value: &Icon) -> Self {
        match value {
            Icon::Unknown => '?',
            Icon::Operational => '.',
            Icon::Damaged => '#',
        }
    }
}

impl TryFrom<char> for Icon {
    type Error = DayError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '?' => Ok(Icon::Unknown),
            '#' => Ok(Icon::Damaged),
            '.' => Ok(Icon::Operational),
            _ => Err(DayError::UnknownSpring(value)),
        }
    }
}

struct SpringList {
    as_icon: Vec<Icon>,
    as_list: Vec<u64>,
}

impl SpringList {
    pub fn get_arrangements(&self) -> u64 {
        RefSpringList::new(&self.as_icon, &self.as_list).start_sub()
    }

    pub fn get_long_arrangements(&self) -> u64 {
        let as_icon = self
            .as_icon
            .iter()
            .copied()
            .chain(std::iter::once(Icon::Unknown))
            .cycle()
            .take((self.as_icon.len() + 1) * 5 - 1)
            .collect_vec();
        let as_list = self
            .as_list
            .iter()
            .copied()
            .cycle()
            .take(self.as_list.len() * 5)
            .collect_vec();
        RefSpringList::new(&as_icon, &as_list).start_sub()
    }
}

struct RefSpringList<'a> {
    as_icon: &'a [Icon],
    as_list: &'a [u64],
    as_list_sum: u64,
    possible: u64,
    minimum: u64,
}

impl<'a> RefSpringList<'a> {
    pub fn new(as_icon: &'a [Icon], as_list: &'a [u64]) -> Self {
        let possible = as_icon
            .iter()
            .filter(|i| !matches!(i, Icon::Operational))
            .count() as u64;
        let minimum = as_icon
            .iter()
            .filter(|i| matches!(i, Icon::Damaged))
            .count() as u64;
        let as_list_sum = as_list.iter().sum();
        Self {
            as_icon,
            as_list,
            as_list_sum,
            possible,
            minimum,
        }
    }

    pub fn start_sub(&self) -> u64 {
        let mut known = HashMap::new();
        self.get_sub(None, &mut known)
    }

    fn pop_icon(&self) -> Self {
        let (possible, minimum) = self.adjust_checks();
        Self {
            as_icon: &self.as_icon[1..],
            as_list: self.as_list,
            as_list_sum: self.as_list_sum,
            possible,
            minimum,
        }
    }

    fn pop_list(&self) -> Self {
        let (possible, minimum) = self.adjust_checks();
        let as_list_sum = self.as_list_sum - self.as_list[0];
        Self {
            as_icon: &self.as_icon[1..],
            as_list: &self.as_list[1..],
            as_list_sum,
            possible,
            minimum,
        }
    }

    #[inline]
    fn adjust_checks(&self) -> (u64, u64) {
        match self.as_icon[0] {
            Icon::Unknown => (self.possible - 1, self.minimum),
            Icon::Operational => (self.possible, self.minimum),
            Icon::Damaged => (self.possible - 1, self.minimum - 1),
        }
    }

    #[inline]
    fn value(&self) -> u64 {
        self.as_list[0]
    }

    fn get_sub(
        &self,
        value: Option<u64>,
        known: &mut HashMap<(usize, u64, Option<u64>), u64>,
    ) -> u64 {
        if self.as_list.is_empty() {
            if self
                .as_icon
                .iter()
                .all(|icon| !matches!(icon, Icon::Damaged))
            {
                return 1;
            } else {
                return 0;
            }
        }
        if self.as_icon.is_empty() {
            if let Some(value) = value {
                if value == self.value() && self.as_list.len() == 1 {
                    return 1;
                }
            };
            return 0;
        }
        let added = value.unwrap_or(0);
        if self.as_list_sum > self.possible + added || self.as_list_sum < self.minimum + added {
            return 0;
        }
        let hash = (self.as_icon.len(), self.as_list_sum, value);
        if let Some(&val) = known.get(&hash) {
            return val;
        }
        let result = match (self.as_icon[0], value) {
            (Icon::Operational, None) => self.pop_icon().get_sub(None, known),
            (Icon::Operational, Some(value)) => {
                if value == self.value() {
                    self.pop_list().get_sub(None, known)
                } else {
                    0
                }
            }
            (Icon::Damaged, None) => self.pop_icon().get_sub(Some(1), known),
            (Icon::Damaged, Some(value)) => {
                if value < self.value() {
                    self.pop_icon().get_sub(Some(value + 1), known)
                } else {
                    0
                }
            }
            (Icon::Unknown, None) => {
                let next = &self.pop_icon();
                next.get_sub(Some(1), known) + next.get_sub(None, known)
            }
            (Icon::Unknown, Some(value)) => match value.cmp(&self.value()) {
                std::cmp::Ordering::Less => self.pop_icon().get_sub(Some(value + 1), known),
                std::cmp::Ordering::Equal => self.pop_list().get_sub(None, known),
                std::cmp::Ordering::Greater => 0,
            },
        };
        known.insert(hash, result);
        result
    }
}

impl FromStr for SpringList {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((as_icon, as_list)) = s.split_once(' ') else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let as_icon = as_icon.chars().map(|p| p.try_into()).try_collect()?;
        let as_list = as_list.split(',').map(|n| n.parse()).try_collect()?;

        Ok(Self { as_icon, as_list })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::days::{read_string, ResultType, UnitResult};

    #[test]
    fn test_part1() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(21);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(525152);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn parse() -> UnitResult {
        let input = "???.### 1,1,3";
        let list: SpringList = input.parse()?;
        assert_eq!(
            list.as_icon,
            [
                Icon::Unknown,
                Icon::Unknown,
                Icon::Unknown,
                Icon::Operational,
                Icon::Damaged,
                Icon::Damaged,
                Icon::Damaged
            ]
        );
        assert_eq!(list.as_list, [1, 1, 3]);
        assert_eq!(list.get_arrangements(), 1);
        assert_eq!(list.get_long_arrangements(), 1);
        Ok(())
    }
    #[test]
    fn parse2() -> UnitResult {
        let input = "?###???????? 3,2,1";
        let list: SpringList = input.parse()?;
        assert_eq!(list.get_arrangements(), 10);
        assert_eq!(list.get_long_arrangements(), 506250);
        Ok(())
    }

    #[test]
    fn parse3() -> UnitResult {
        let input = "????.######..#####. 1,6,5";
        let list: SpringList = input.parse()?;
        assert_eq!(list.get_arrangements(), 4);
        assert_eq!(list.get_long_arrangements(), 2500);
        Ok(())
    }
}
