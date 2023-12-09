use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{num, str::FromStr};

const DAY_NUMBER: DayType = 9;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let result = input
            .lines()
            .map(|line| line.parse::<Sequence>())
            .fold_ok(0, |acc, seq| acc + seq.find_next().1)?;
        Ok(result.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let result = input
            .lines()
            .map(|line| line.parse::<Sequence>())
            .fold_ok(0, |acc, seq| acc + seq.find_next().0)?;
        Ok(result.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
}

struct Sequence {
    values: Vec<i64>,
}

impl Sequence {
    pub fn find_next(&self) -> (i64, i64) {
        let mut first = vec![];
        let mut last = vec![];
        let mut values = self.values.clone();

        while !values.iter().all(|&v| v == 0) {
            first.push(*values.first().unwrap());
            last.push(*values.last().unwrap());
            values = values
                .into_iter()
                .tuple_windows()
                .map(|(f, s)| s - f)
                .collect_vec();
        }

        (
            first
                .into_iter()
                .rev()
                .fold(0, |new_first, diff_first| diff_first - new_first),
            last.into_iter().sum::<i64>(),
        )
    }
}

impl FromStr for Sequence {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s
            .split_ascii_whitespace()
            .map(|num| num.parse())
            .try_collect()?;
        Ok(Self { values })
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
        let expected = ResultType::Integer(114);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(2);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn parse() -> UnitResult {
        let input = "10 13 16 21 30 45";
        let seq: Sequence = input.parse()?;
        assert_eq!(seq.find_next(), (5, 68));

        let input = "0 3 6 9 12 15";
        let seq: Sequence = input.parse()?;
        assert_eq!(seq.find_next(), (-3, 18));

        let input = "1 3 6 10 15 21";
        let seq: Sequence = input.parse()?;
        assert_eq!(seq.find_next(), (0, 28));

        Ok(())
    }
}
