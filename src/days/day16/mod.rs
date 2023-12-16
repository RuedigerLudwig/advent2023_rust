use super::{DayTrait, DayType, RResult};
use crate::common::{direction::Direction, pos2::Pos2};
use colored::Colorize;
use itertools::Itertools;
use std::{collections::HashMap, num, str::FromStr};

const DAY_NUMBER: DayType = 16;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let contraption: Contraption = input.parse()?;
        Ok(contraption.follow_beam().into())
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
    #[error("Unknown Mirror: {0}")]
    UnknownMirror(char),
}

enum Mirror {
    None,
    Horizontal,
    Vertical,
    UpRight,
    UpLeft,
}

impl TryFrom<char> for Mirror {
    type Error = DayError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Mirror::None),
            '-' => Ok(Mirror::Horizontal),
            '|' => Ok(Mirror::Vertical),
            '/' => Ok(Mirror::UpRight),
            '\\' => Ok(Mirror::UpLeft),
            _ => Err(DayError::UnknownMirror(value)),
        }
    }
}

struct Beam {
    pos: Pos2<usize>,
    direction: Direction,
    visited: Vec<Vec<Vec<Pos2<usize>>>>,
    splits: Vec<(Pos2<usize>, Direction)>,
}

impl Beam {
    pub fn new(start: Pos2<usize>, direction: Direction) -> Self {
        Self {
            pos: start,
            direction,
            visited: vec![],
            splits: vec![],
        }
    }
}

struct Contraption {
    mirrors: Vec<Vec<Mirror>>,
}

impl Contraption {
    pub fn follow_beam(&self) -> usize {
        let mut known_splits = HashMap::new();
        let mut beam = Beam::new(Pos2::new(0, 0), Direction::East);
        let mut touched = vec![vec![vec![]; self.mirrors[0].len()]; self.mirrors.len()];

        loop {
            match beam.pos.safe_matrix_get(&self.mirrors).unwrap() {
                Mirror::None => {}
                Mirror::Horizontal => {
                    if beam.direction.is_vertical() {
                        if let Some(touched) = known_splits.get(&(beam.pos, beam.direction)) {
                            todo!()
                        }
                        beam.direction = beam.direction.turn_right();
                        let other = beam.direction.turn_back();
                        if let Some(next) = beam.pos.safe_matrix_add(&self.mirrors, other) {
                            beam.splits.push((next, other));
                        }
                    }
                }
                Mirror::Vertical => {
                    if beam.direction.is_horizontal() {
                        beam.direction = beam.direction.turn_right();
                        let other = beam.direction.turn_back();
                        if let Some(next) = beam.pos.safe_matrix_add(&self.mirrors, other) {
                            beam.splits.push((next, other));
                        }
                    }
                }
                Mirror::UpRight => {
                    if beam.direction.is_horizontal() {
                        beam.direction = beam.direction.turn_left();
                    } else {
                        beam.direction = beam.direction.turn_right();
                    }
                }
                Mirror::UpLeft => {
                    if beam.direction.is_vertical() {
                        beam.direction = beam.direction.turn_left();
                    } else {
                        beam.direction = beam.direction.turn_right();
                    }
                }
            }
            let mut prev = beam.pos.safe_matrix_get(&touched).unwrap().clone();
            if prev.contains(&beam.direction) {
                beam.pos.safe_matrix_set(&mut touched, prev);
                break;
            }
            prev.push(beam.direction);
            beam.pos.safe_matrix_set(&mut touched, prev);
            let Some(next_pos) = beam.pos.safe_matrix_add(&self.mirrors, beam.direction) else {
                break;
            };
            beam.pos = next_pos;
        }
        touched_count
    }
}

impl FromStr for Contraption {
    type Err = DayError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            mirrors: input
                .lines()
                .map(|row| row.chars().map(|c| c.try_into()).try_collect())
                .try_collect()?,
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
        let expected = ResultType::Integer(46);
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
