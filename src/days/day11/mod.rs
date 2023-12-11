use crate::common::pos2::Pos2;

use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{num, str::FromStr};

const DAY_NUMBER: DayType = 11;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let map: GalaxyMap = input.parse()?;
        Ok(map.sum_young().into())
    }

    fn part2(&self, input: &str) -> RResult {
        let map: GalaxyMap = input.parse()?;
        Ok(map.sum_old(1_000_000).into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Illegal char: {0}")]
    IllegalChar(char),
}

struct GalaxyMap {
    galaxies: Vec<Pos2<usize>>,
    empty_rows: Vec<usize>,
    empty_cols: Vec<usize>,
}

impl FromStr for GalaxyMap {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(DayError::ParseError(s.to_owned()));
        }

        let galaxies: Vec<_> = s
            .lines()
            .enumerate()
            .flat_map(|(y, row)| {
                row.chars()
                    .enumerate()
                    .filter_map(move |(x, point)| match point {
                        '#' => Some(Ok(Pos2::new(x, y))),
                        '.' => None,
                        _ => Some(Err(DayError::IllegalChar(point))),
                    })
            })
            .try_collect()?;

        let empty_rows = s
            .lines()
            .enumerate()
            .filter(|(_, row)| row.chars().all(|c| c == '.'))
            .map(|(y, _)| y)
            .collect_vec();

        let all_cols = galaxies.iter().map(|galaxy| galaxy.x()).collect_vec();
        let max_col = all_cols.iter().max().copied().unwrap_or(0);
        let empty_cols = (0..max_col)
            .filter(|col| !all_cols.contains(col))
            .collect_vec();

        Ok(Self {
            empty_cols,
            empty_rows,
            galaxies,
        })
    }
}

impl GalaxyMap {
    #[inline]
    pub fn sum_young(&self) -> usize {
        self.sum_old(2)
    }

    #[inline]
    pub fn sum_old(&self, expansion: usize) -> usize {
        assert!(expansion > 0);
        self.distances(expansion).into_iter().sum()
    }

    fn count_free_space(from: usize, to: usize, lst: &[usize]) -> usize {
        let mn = from.min(to);
        let mx = from.max(to);
        (mn..mx).filter(|num| lst.contains(num)).count()
    }

    pub fn distances(&self, expansion: usize) -> Vec<usize> {
        let factor = expansion - 1;
        self.galaxies
            .iter()
            .tuple_combinations()
            .map(|(fst, snd)| {
                let dist = fst.taxicab_between(*snd);
                let add_row = Self::count_free_space(fst.y(), snd.y(), &self.empty_rows) * factor;
                let add_col = Self::count_free_space(fst.x(), snd.x(), &self.empty_cols) * factor;
                dist + add_row + add_col
            })
            .collect_vec()
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
        let expected = ResultType::Integer(374);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn parse() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;

        let map: GalaxyMap = input.parse()?;

        assert_eq!(map.galaxies.len(), 9);
        assert_eq!(map.empty_rows, [3, 7]);
        assert_eq!(map.empty_cols, [2, 5, 8]);
        Ok(())
    }

    #[test]
    fn expand() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;

        let map: GalaxyMap = input.parse()?;
        assert_eq!(map.sum_young(), 374);
        assert_eq!(map.sum_old(10), 1030);
        assert_eq!(map.sum_old(100), 8410);

        Ok(())
    }
}
