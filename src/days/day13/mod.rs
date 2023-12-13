use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{num, str::FromStr};

const DAY_NUMBER: DayType = 13;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let pl: PatternList = input.parse()?;
        Ok(pl.get_evaluation(0).into())
    }

    fn part2(&self, input: &str) -> RResult {
        let pl: PatternList = input.parse()?;
        Ok(pl.get_evaluation(1).into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Pattern is not a rectangle")]
    NotAReactangle,
    #[error("Unknown Char: {0}")]
    UnknownChar(char),
}

struct Pattern {
    dots: Vec<Vec<bool>>,
}

impl Pattern {
    fn count_smudges(fst: &[bool], snd: &[bool]) -> usize {
        fst.iter()
            .zip(snd.iter())
            .filter(|(fst, snd)| fst != snd)
            .count()
    }

    fn check_horizontal(&self, expected_smudges: usize) -> Option<usize> {
        self.dots
            .iter()
            .enumerate()
            .tuple_windows()
            .filter_map(|((pos, fst), (_, snd))| {
                if Self::count_smudges(fst, snd) <= expected_smudges {
                    Some(pos)
                } else {
                    None
                }
            })
            .find_map(|row| {
                let end = row.min(self.dots.len() - (row + 2));
                let smudges: usize = (0..=end)
                    .map(|r| Self::count_smudges(&self.dots[row - r], &self.dots[row + r + 1]))
                    .sum();
                if smudges == expected_smudges {
                    Some(row + 1)
                } else {
                    None
                }
            })
    }

    fn check_vertical(&self, expected_smudges: usize) -> Option<usize> {
        self.transpose().check_horizontal(expected_smudges)
    }

    fn transpose(&self) -> Self {
        Self {
            dots: (0..self.dots[0].len())
                .map(|x| (0..self.dots.len()).map(|y| self.dots[y][x]).collect_vec())
                .collect_vec(),
        }
    }

    fn new(lines: &mut std::str::Lines<'_>) -> Result<Option<Self>, DayError> {
        let dots: Vec<Vec<bool>> = lines
            .take_while(|line| !line.is_empty())
            .map(|line| {
                line.chars()
                    .map(|d| match d {
                        '#' => Ok(true),
                        '.' => Ok(false),
                        _ => Err(DayError::UnknownChar(d)),
                    })
                    .try_collect()
            })
            .try_collect()?;

        if dots.is_empty() {
            Ok(None)
        } else if !dots.iter().map(|row| row.len()).all_equal() {
            Err(DayError::NotAReactangle)
        } else {
            Ok(Some(Self { dots }))
        }
    }
}

struct PatternList {
    list: Vec<Pattern>,
}

impl FromStr for PatternList {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let mut list = vec![];
        while let Some(pattern) = Pattern::new(&mut lines)? {
            list.push(pattern)
        }
        Ok(Self { list })
    }
}

impl PatternList {
    pub fn get_evaluation(&self, expected_smudges: usize) -> usize {
        self.list
            .iter()
            .filter_map(|pattern| {
                pattern
                    .check_horizontal(expected_smudges)
                    .map(|h| h * 100)
                    .or_else(|| pattern.check_vertical(expected_smudges))
            })
            .sum()
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
        let expected = ResultType::Integer(405);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(400);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn parse() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;

        let pl: PatternList = input.parse()?;
        assert_eq!(pl.list.len(), 2);
        assert_eq!(pl.list[0].check_horizontal(0), None);
        assert_eq!(pl.list[0].check_vertical(0), Some(5));
        assert_eq!(pl.list[1].check_horizontal(0), Some(4));
        assert_eq!(pl.list[1].check_vertical(0), None);

        Ok(())
    }

    #[test]
    fn parse_smack() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;

        let pl: PatternList = input.parse()?;
        assert_eq!(pl.list.len(), 2);
        assert_eq!(pl.list[0].check_horizontal(1), Some(3));
        assert_eq!(pl.list[0].check_vertical(1), None);
        assert_eq!(pl.list[1].check_horizontal(1), Some(1));
        assert_eq!(pl.list[1].check_vertical(1), None);

        Ok(())
    }
}
