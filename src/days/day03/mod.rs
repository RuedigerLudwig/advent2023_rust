use super::{DayTrait, DayType, RResult};
use crate::common::pos2::Pos2;
use itertools::Itertools;
use std::{num, str::FromStr};

const DAY_NUMBER: DayType = 3;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let schema: Schema = input.parse()?;
        let result: i64 = schema.filter_adjacent().into_iter().sum();
        Ok(result.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let schema: Schema = input.parse()?;
        let result: i64 = schema.get_gears().into_iter().sum();
        Ok(result.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
}

#[derive(Debug, PartialEq, Eq)]
enum Information {
    Symbol(char, Pos2<usize>),
    Number(i64, Pos2<usize>, usize),
}

struct Schema {
    information: Vec<Information>,
}

impl FromStr for Schema {
    type Err = DayError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let information = input
            .lines()
            .enumerate()
            .flat_map(|(y, line)| {
                let (line, _) = line.chars().enumerate().fold(
                    (vec![], false),
                    |(mut row, in_number): (Vec<Information>, bool), (x, c)| match (
                        c,
                        row.last(),
                        in_number,
                    ) {
                        ('0'..='9', Some(Information::Number(num, start, len)), true) => {
                            let information = Information::Number(
                                num * 10 + c.to_digit(10).unwrap() as i64,
                                *start,
                                len + 1,
                            );
                            let last_pos = row.len() - 1;
                            row[last_pos] = information;
                            (row, true)
                        }
                        ('0'..='9', _, _) => {
                            let information = Information::Number(
                                c.to_digit(10).unwrap() as i64,
                                Pos2::new(x, y),
                                1,
                            );
                            row.push(information);
                            (row, true)
                        }
                        ('.', _, _) => (row, false),
                        (_, _, _) => {
                            row.push(Information::Symbol(c, Pos2::new(x, y)));
                            (row, false)
                        }
                    },
                );
                line
            })
            .collect_vec();
        Ok(Self { information })
    }
}

impl Schema {
    fn find_symbol(&self, start: &Pos2<usize>, len: usize) -> bool {
        self.information.iter().any(|info| match info {
            Information::Symbol(_, pos) => {
                (start.y().saturating_sub(1)..=start.y() + 1).contains(&pos.y())
                    && (start.x().saturating_sub(1)..=start.x() + len).contains(&pos.x())
            }
            Information::Number(_, _, _) => false,
        })
    }

    pub fn filter_adjacent(&self) -> Vec<i64> {
        self.information
            .iter()
            .filter_map(|info| match info {
                Information::Symbol(_, _) => None,
                Information::Number(num, start, len) => {
                    self.find_symbol(start, *len).then_some(*num)
                }
            })
            .collect_vec()
    }

    pub fn check_gear(&self, pos: &Pos2<usize>) -> Option<i64> {
        let gear = self
            .information
            .iter()
            .filter_map(|info| match info {
                Information::Number(num, start, len) => {
                    if (start.y().saturating_sub(1)..=start.y() + 1).contains(&pos.y())
                        && (start.x().saturating_sub(1)..=start.x() + len).contains(&pos.x())
                    {
                        Some(*num)
                    } else {
                        None
                    }
                }
                Information::Symbol(_, _) => None,
            })
            .collect_vec();

        if gear.len() == 2 {
            Some(gear[0] * gear[1])
        } else {
            None
        }
    }

    pub fn get_gears(&self) -> Vec<i64> {
        self.information
            .iter()
            .filter_map(|info| match info {
                Information::Symbol('*', pos) => self.check_gear(pos),
                _ => None,
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
        let expected = ResultType::Integer(4361);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(467835);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn read_input() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = [
            Information::Number(467, Pos2::new(0, 0), 3),
            Information::Number(114, Pos2::new(5, 0), 3),
            Information::Symbol('*', Pos2::new(3, 1)),
            Information::Number(35, Pos2::new(2, 2), 2),
            Information::Number(633, Pos2::new(6, 2), 3),
            Information::Symbol('#', Pos2::new(6, 3)),
        ];
        let result: Schema = input.parse()?;
        assert_eq!(result.information.len(), 16);
        assert_eq!(result.information[0..6], expected);

        Ok(())
    }

    #[test]
    fn find_numbers() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let result: Schema = input.parse()?;
        let expected = [467, 35, 633, 617, 592, 755, 664, 598];
        assert_eq!(result.filter_adjacent(), expected);

        Ok(())
    }

    #[test]
    fn find_gears() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let schema: Schema = input.parse()?;
        let expected = [16345, 451490];
        assert_eq!(schema.get_gears(), expected);

        Ok(())
    }
}
