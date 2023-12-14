use crate::common::{direction::Direction, pos2::Pos2};

use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{collections::HashMap, str::FromStr};

const DAY_NUMBER: DayType = 14;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let mut field: Platform = input.parse()?;
        field.roll_to(Direction::North);

        Ok(field.calc_load().into())
    }

    fn part2(&self, input: &str) -> RResult {
        let mut field: Platform = input.parse()?;

        Ok(field.northern_load_after(1_000_000_000).into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Unknown Rock {0}")]
    UnknonwRock(char),
    #[error("Need a reactangle platform")]
    NeedRectanglePlatform,
    #[error("Platform must not be empty")]
    EmptyPlatform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Rock {
    Empty,
    Round,
    Cubed,
}

impl TryFrom<char> for Rock {
    type Error = DayError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'O' => Ok(Rock::Round),
            '#' => Ok(Rock::Cubed),
            '.' => Ok(Rock::Empty),
            _ => Err(DayError::UnknonwRock(value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Platform {
    rocks: Vec<Vec<Rock>>,
}

impl FromStr for Platform {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rocks: Vec<Vec<Rock>> = s
            .lines()
            .map(|row| row.chars().map(|c| c.try_into()).try_collect())
            .try_collect()?;
        if rocks.is_empty() || rocks[0].is_empty() {
            return Err(DayError::EmptyPlatform);
        }
        if !rocks.iter().map(|row| row.len()).all_equal() {
            return Err(DayError::NeedRectanglePlatform);
        }
        Ok(Self { rocks })
    }
}
impl Platform {
    fn one_cycle(&mut self) {
        self.roll_to(Direction::North);
        self.roll_to(Direction::West);
        self.roll_to(Direction::South);
        self.roll_to(Direction::East);
    }

    fn northern_load_after(&mut self, cycles: usize) -> usize {
        let mut seen: HashMap<Self, usize> = HashMap::new();
        let mut round = 0;
        while round < cycles {
            self.one_cycle();
            if let Some(&last_seen) = seen.get(self) {
                let diff = round - last_seen;
                round += (cycles - round) / diff * diff + 1;
                break;
            }
            seen.insert(self.clone(), round);
            round += 1;
        }
        for _ in round..cycles {
            self.one_cycle();
        }
        self.calc_load()
    }

    fn roll_to(&mut self, direction: Direction) {
        let row_diw = direction.turn_right();
        let search_dir = direction.turn_back();
        let mut row_start = Some(match direction {
            Direction::North => Pos2::new(0, 0),
            Direction::West => Pos2::new(0, self.rocks.len() - 1),
            Direction::South => Pos2::new(self.rocks[0].len() - 1, self.rocks.len() - 1),
            Direction::East => Pos2::new(self.rocks[0].len() - 1, 0),
        });
        while let Some(row) = row_start {
            let mut first_block = Some(row);
            while let Some(mut fb) = first_block {
                if matches!(fb.safe_matrix_get(&self.rocks), Some(Rock::Empty)) {
                    let mut second_block = fb;
                    while let Some((next_sb, &item)) =
                        second_block.safe_matrix_add_and_get(&self.rocks, search_dir)
                    {
                        match item {
                            Rock::Empty => {
                                second_block = next_sb;
                            }
                            Rock::Cubed => {
                                fb = next_sb;
                                break;
                            }
                            Rock::Round => {
                                fb.safe_matrix_set(&mut self.rocks, Rock::Round);
                                next_sb.safe_matrix_set(&mut self.rocks, Rock::Empty);
                                break;
                            }
                        }
                    }
                }
                first_block = fb.safe_matrix_add(&self.rocks, search_dir);
            }
            row_start = row.safe_matrix_add(&self.rocks, row_diw);
        }
    }

    fn calc_load(&self) -> usize {
        let len = self.rocks[0].len();
        self.rocks
            .iter()
            .map(|row| {
                row.iter()
                    .filter(|item| matches!(item, Rock::Round))
                    .count()
            })
            .enumerate()
            .map(|(row, rolling)| (len - row) * rolling)
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
        let expected = ResultType::Integer(136);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(64);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn cycle() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let mut field: Platform = input.parse()?;

        field.one_cycle();

        let input = read_string(day.get_day_number(), "expected01.txt")?;
        let expected: Platform = input.parse()?;

        assert_eq!(field, expected);

        Ok(())
    }
}
