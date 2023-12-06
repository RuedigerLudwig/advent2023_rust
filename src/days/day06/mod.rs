use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{num, str::FromStr};

const DAY_NUMBER: DayType = 6;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let table: Table = input.parse()?;
        let result: u64 = table.count_winning().product();
        Ok(result.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let table: Race = input.parse()?;
        let result = table.count_winning();
        Ok(result.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Not same amount of times and distances")]
    NotEqualLength,
}

#[derive(Debug, PartialEq, Eq)]
struct Race {
    time: u64,
    distance: u64,
}

impl Race {
    #[inline]
    pub fn distance(&self, hold_time: u64) -> u64 {
        (self.time - hold_time) * hold_time
    }

    #[allow(dead_code)]
    #[inline]
    pub fn get_distances(&self) -> impl Iterator<Item = u64> + '_ {
        (0..=self.time).map(|hold| self.distance(hold))
    }

    fn find_winning_hold(&self, mut min_time: u64, mut max_time: u64, from_shorter: bool) -> u64 {
        while min_time + 1 < max_time {
            let middle = (min_time + max_time) / 2;
            let distance = self.distance(middle);
            if (distance <= self.distance) == from_shorter {
                min_time = middle;
            } else {
                max_time = middle;
            }
        }

        max_time
    }

    #[inline]
    pub fn count_winning(&self) -> u64 {
        let start = self.find_winning_hold(0, self.time / 2, true);
        let end = self.find_winning_hold(self.time / 2, self.time, false);
        end - start
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Table {
    races: Vec<Race>,
}

impl Table {
    pub fn count_winning(&self) -> impl Iterator<Item = u64> + '_ {
        self.races.iter().map(|race| race.count_winning())
    }
}

impl FromStr for Table {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let Some(times) = lines.next() else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let Some(times) = times.strip_prefix("Time:") else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let times: Vec<_> = times
            .split_ascii_whitespace()
            .map(|time| time.parse::<u64>())
            .try_collect()?;

        let Some(distances) = lines.next() else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let Some(distances) = distances.strip_prefix("Distance:") else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let distances: Vec<_> = distances
            .split_ascii_whitespace()
            .map(|time| time.parse::<u64>())
            .try_collect()?;

        if times.len() != distances.len() {
            return Err(DayError::NotEqualLength);
        }

        let races = times
            .into_iter()
            .zip(distances)
            .map(|(time, distance)| Race { time, distance })
            .collect_vec();

        Ok(Self { races })
    }
}

impl FromStr for Race {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let Some(time) = lines.next() else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let Some(times) = time.strip_prefix("Time:") else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let time = times.replace(' ', "");
        let time = time.parse()?;

        let Some(distance) = lines.next() else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let Some(distance) = distance.strip_prefix("Distance:") else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let distance = distance.replace(' ', "");
        let distance = distance.parse()?;

        Ok(Self { time, distance })
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
        let expected = ResultType::Integer(288);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(71503);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn parse() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let table: Table = input.parse()?;
        let expected = [
            Race {
                time: 7,
                distance: 9,
            },
            Race {
                time: 15,
                distance: 40,
            },
            Race {
                time: 30,
                distance: 200,
            },
        ];
        assert_eq!(table.races, expected);

        Ok(())
    }

    #[test]
    fn distances() {
        let race = Race {
            time: 7,
            distance: 9,
        };
        let expected = [0, 6, 10, 12, 12, 10, 6, 0];
        assert_eq!(race.get_distances().collect_vec(), expected);

        assert_eq!(race.count_winning(), 4);
    }

    #[test]
    fn all_distances() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let table: Table = input.parse()?;
        assert_eq!(table.count_winning().collect_vec(), [4, 8, 9]);

        Ok(())
    }
    #[test]
    fn parse_real() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let table: Race = input.parse()?;
        let expected = Race {
            time: 71530,
            distance: 940200,
        };
        assert_eq!(table, expected);
        Ok(())
    }
}
