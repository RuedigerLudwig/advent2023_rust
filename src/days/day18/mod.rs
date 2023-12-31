use super::{DayTrait, DayType, RResult};
use crate::common::{direction::Direction, pos2::Pos2, turn::Turn};
use itertools::Itertools;
use num_traits::Zero;
use std::num;

const DAY_NUMBER: DayType = 18;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let lagoon = Lagoon::from_simple(input)?;
        let steps = lagoon.pool_size();
        Ok(steps.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let lagoon = Lagoon::from_coded(input)?;
        let steps = lagoon.pool_size();
        Ok(steps.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Illegal Turn")]
    IllegalTurn,
    #[error("Lagoon instructions do not loop back to start")]
    DoesNotLoopBack,
    #[error("Loop must not be empty")]
    LoopMustBeAtLeast4,
}

#[derive(Debug)]
struct Instruction {
    direction: Direction,
    steps: i64,
}

impl Instruction {
    pub fn from_simple(input: &str) -> Result<Self, DayError> {
        let mut parts = input.split_ascii_whitespace();
        let Some(direction) = parts.next() else {
            return Err(DayError::ParseError(input.to_owned()));
        };
        let direction = match direction {
            "U" => Direction::North,
            "R" => Direction::East,
            "D" => Direction::South,
            "L" => Direction::West,
            _ => return Err(DayError::ParseError(input.to_owned())),
        };
        let Some(steps) = parts.next() else {
            return Err(DayError::ParseError(input.to_owned()));
        };
        Ok(Self {
            direction,
            steps: steps.parse()?,
        })
    }

    pub fn from_coded(input: &str) -> Result<Self, DayError> {
        let Some((_, hex)) = input.split_once('#') else {
            return Err(DayError::ParseError(input.to_owned()));
        };
        let Some(color) = hex.strip_suffix(')') else {
            return Err(DayError::ParseError(input.to_owned()));
        };
        let direction = match color.chars().nth(5) {
            Some('0') => Direction::East,
            Some('1') => Direction::South,
            Some('2') => Direction::West,
            Some('3') => Direction::North,
            _ => return Err(DayError::ParseError(input.to_owned())),
        };
        let steps = color
            .chars()
            .take(5)
            .fold(0, |s, c| s * 16 + c.to_digit(16).unwrap() as i64);
        Ok(Self { direction, steps })
    }

    #[inline]
    pub fn direction(&self) -> Direction {
        self.direction
    }

    #[allow(dead_code)]
    #[inline]
    pub fn steps(&self) -> i64 {
        self.steps
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    pub fn walk(&self, mut pos: Pos2<i64>) -> Pos2<i64> {
        pos += match self.direction {
            Direction::East => Pos2::new_x(self.steps),
            Direction::North => Pos2::new_y(-self.steps),
            Direction::West => Pos2::new_x(-self.steps),
            Direction::South => Pos2::new_y(self.steps),
        };
        pos
    }
}

struct Lagoon {
    instructions: Vec<Instruction>,
}

impl Lagoon {
    pub fn new(instructions: Vec<Instruction>) -> Result<Self, DayError> {
        let mut lagoon = Self { instructions };
        lagoon.normalize()?;
        Ok(lagoon)
    }

    pub fn from_simple(input: &str) -> Result<Self, DayError> {
        Self::new(input.lines().map(Instruction::from_simple).try_collect()?)
    }

    pub fn from_coded(input: &str) -> Result<Self, DayError> {
        Self::new(input.lines().map(Instruction::from_coded).try_collect()?)
    }

    fn does_loop_back(&self) -> bool {
        self.instructions
            .iter()
            .fold(Pos2::zero(), |pos, instruction| instruction.walk(pos))
            .is_zero()
    }

    fn get_inside_turn(&self) -> Result<Turn, DayError> {
        if self.instructions.len() < 4 {
            return Err(DayError::LoopMustBeAtLeast4);
        }
        if !self.does_loop_back() {
            return Err(DayError::DoesNotLoopBack);
        }

        let turns = self.instructions.iter().circular_tuple_windows().try_fold(
            0,
            |turns, (curr, next)| match curr.direction().get_turn(next.direction()) {
                Turn::Left => Ok(turns - 1),
                Turn::Right => Ok(turns + 1),
                _ => Err(DayError::IllegalTurn),
            },
        )?;

        if turns < 0 {
            Ok(Turn::Left)
        } else {
            Ok(Turn::Right)
        }
    }

    pub fn pool_size(&self) -> i64 {
        let (area, bars) = self
            .instructions
            .iter()
            .circular_tuple_windows()
            .scan(Pos2::new(6, 0), |pos, (prev, curr, next)| {
                let prev_north = prev.direction() == Direction::North;
                let next_north = next.direction() == Direction::North;
                let prev_pos = *pos;
                *pos = curr.walk(prev_pos);

                let operation = match curr.direction() {
                    Direction::East => Some(LagoonOperation::add(LagoonBar::new(
                        pos.y(),
                        prev_pos.x() + if prev_north { 0 } else { 1 },
                        pos.x() - if next_north { 1 } else { 0 },
                    ))),
                    Direction::West => Some(LagoonOperation::sub(LagoonBar::new(
                        pos.y(),
                        pos.x() + if next_north { 0 } else { 1 },
                        prev_pos.x() - if prev_north { 1 } else { 0 },
                    ))),
                    Direction::North | Direction::South => None,
                };
                Some(operation)
            })
            .flatten()
            .sorted()
            .fold(
                (0, vec![]),
                |(mut area, mut bars): (i64, Vec<LagoonBar>), operation| {
                    if operation.is_add() {
                        bars.push(operation.bar);
                        (area, bars)
                    } else {
                        let mut next_bars = vec![];
                        for old_bar in bars {
                            if let Some((added_area, mut inter_bars)) =
                                old_bar.cut_out(&operation.bar)
                            {
                                area += added_area;
                                next_bars.append(&mut inter_bars);
                            } else {
                                next_bars.push(old_bar);
                            }
                        }
                        (area, next_bars)
                    }
                },
            );
        if !bars.is_empty() {
            panic!("Programming error");
        }
        area
    }

    fn normalize(&mut self) -> Result<(), DayError> {
        let inside = self.get_inside_turn()?;
        if inside == Turn::Left {
            self.instructions
                .iter_mut()
                .for_each(|instruction| match instruction.direction() {
                    Direction::East => instruction.set_direction(Direction::West),
                    Direction::West => instruction.set_direction(Direction::East),
                    Direction::North | Direction::South => {}
                })
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LagoonBar {
    y: i64,
    x1: i64,
    x2: i64,
}

impl LagoonBar {
    fn new(y: i64, x1: i64, x2: i64) -> LagoonBar {
        Self { y, x1, x2 }
    }

    fn cut_out(&self, other: &LagoonBar) -> Option<(i64, Vec<LagoonBar>)> {
        if self.x1 > other.x2 || self.x2 < other.x1 {
            return None;
        }
        let area = (self.x2 - self.x1 + 1) * (other.y - self.y + 1);

        let mut bars = vec![];
        if other.x1 > self.x1 {
            bars.push(LagoonBar::new(other.y + 1, self.x1, other.x1 - 1))
        }
        if other.x2 < self.x2 {
            bars.push(LagoonBar::new(other.y + 1, other.x2 + 1, self.x2))
        }

        Some((area, bars))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LagoonOperation {
    bar: LagoonBar,
    add: bool,
}

impl LagoonOperation {
    pub fn add(bar: LagoonBar) -> Self {
        Self { bar, add: true }
    }

    pub fn sub(bar: LagoonBar) -> Self {
        Self { bar, add: false }
    }

    pub fn is_add(&self) -> bool {
        self.add
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
        let expected = ResultType::Integer(62);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(952408144115);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn parse() -> UnitResult {
        let input = "R 6 (#70c710)";

        let simple = Instruction::from_simple(input)?;
        assert_eq!(simple.direction(), Direction::East);
        assert_eq!(simple.steps(), 6);

        let real = Instruction::from_coded(input)?;
        assert_eq!(real.direction(), Direction::East);
        assert_eq!(real.steps(), 461937);

        Ok(())
    }

    #[test]
    fn walk() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;

        let lagoon = Lagoon::from_simple(&input)?;
        assert_eq!(lagoon.pool_size(), 62);

        let lagoon = Lagoon::from_coded(&input)?;
        assert_eq!(lagoon.pool_size(), 952408144115);

        Ok(())
    }
}
