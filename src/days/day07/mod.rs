use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{num, str::FromStr};

const DAY_NUMBER: DayType = 7;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        Ok(().into())
    }

    fn part2(&self, input: &str) -> RResult {
        Ok(().into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Not a card")]
    NotACard(char),
}

struct Hand {
    cards: Vec<u8>,
    value: u64,
}

impl Hand {
    pub fn get_rank(&self) -> u64 {
        let x = self.cards.iter().sorted().group_by()
    }
}

impl FromStr for Hand {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((hand, value)) = s.split_once(' ') else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let cards: Vec<u8> = hand
            .chars()
            .map(|c| match c {
                '2'..='9' => Ok(c.to_digit(10).map(|d| d as u8).unwrap()),
                'J' => Ok(10),
                'Q' => Ok(11),
                'K' => Ok(12),
                'A' => Ok(13),
                _ => Err(DayError::NotACard(c)),
            })
            .try_collect()?;
        if cards.len() != 5 {
            return Err(DayError::ParseError(s.to_owned()));
        }
        Ok(Self {
            cards,
            value: value.parse()?,
        })
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
        let expected = ResultType::Nothing;
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Nothing;
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }
}
