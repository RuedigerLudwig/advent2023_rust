use crate::common::{direction::Direction, pos2::Pos2, turn::Turn};

use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{num, str::FromStr};

const DAY_NUMBER: DayType = 10;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let map: PipeMap = input.parse()?;
        let length = map.analyze_loop()?.steps / 2;
        Ok(length.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let map: PipeMap = input.parse()?;
        let enclosed = map.count_enclosed()?;
        Ok(enclosed.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Illegal Pipe-Char: {0}")]
    IllegalChar(char),
    #[error("An emopty map is not allowed")]
    EmptyMapNoAllowed,
    #[error("A map must be rectangle")]
    MapMustBerectangle,
    #[error("Need exactly one start")]
    NeedExactlyOneStart,
    #[error("No loop was found")]
    NoLoopFound,
    #[error("Illegal Turns")]
    IllegalTurns,
    #[error("Can't turn back in loop")]
    CantTurnBack,
}

#[derive(Debug, Clone, Copy)]
enum Pipe {
    Start,
    NorthSouth,
    EastWest,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
    Ground,
}

impl TryFrom<char> for Pipe {
    type Error = DayError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'S' => Ok(Self::Start),
            '|' => Ok(Self::NorthSouth),
            '-' => Ok(Self::EastWest),
            'L' => Ok(Self::NorthEast),
            'J' => Ok(Self::NorthWest),
            '7' => Ok(Self::SouthWest),
            'F' => Ok(Self::SouthEast),
            '.' => Ok(Self::Ground),
            _ => Err(DayError::IllegalChar(value)),
        }
    }
}

impl Pipe {
    pub fn exit(&self, from: Direction) -> Option<Direction> {
        match (self, from) {
            (Pipe::NorthSouth, Direction::North) => Some(Direction::South),
            (Pipe::NorthSouth, Direction::South) => Some(Direction::North),
            (Pipe::EastWest, Direction::East) => Some(Direction::West),
            (Pipe::EastWest, Direction::West) => Some(Direction::East),
            (Pipe::NorthEast, Direction::East) => Some(Direction::North),
            (Pipe::NorthEast, Direction::North) => Some(Direction::East),
            (Pipe::NorthWest, Direction::North) => Some(Direction::West),
            (Pipe::NorthWest, Direction::West) => Some(Direction::North),
            (Pipe::SouthEast, Direction::East) => Some(Direction::South),
            (Pipe::SouthEast, Direction::South) => Some(Direction::East),
            (Pipe::SouthWest, Direction::West) => Some(Direction::South),
            (Pipe::SouthWest, Direction::South) => Some(Direction::West),
            _ => None,
        }
    }

    pub fn get_inside_directions(&self, dir: Direction, turn: Turn) -> Vec<Direction> {
        match (self, dir, turn) {
            (Pipe::NorthSouth, Direction::North, Turn::Right)
            | (Pipe::NorthSouth, Direction::South, Turn::Left) => vec![Direction::East],

            (Pipe::NorthSouth, Direction::North, Turn::Left)
            | (Pipe::NorthSouth, Direction::South, Turn::Right) => vec![Direction::West],

            (Pipe::EastWest, Direction::West, Turn::Left)
            | (Pipe::EastWest, Direction::East, Turn::Right) => vec![Direction::South],

            (Pipe::EastWest, Direction::East, Turn::Left)
            | (Pipe::EastWest, Direction::West, Turn::Right) => vec![Direction::North],

            (Pipe::NorthEast, Direction::East, Turn::Right)
            | (Pipe::NorthEast, Direction::North, Turn::Left) => {
                vec![Direction::South, Direction::West]
            }

            (Pipe::NorthWest, Direction::West, Turn::Left)
            | (Pipe::NorthWest, Direction::North, Turn::Right) => {
                vec![Direction::South, Direction::East]
            }

            (Pipe::SouthEast, Direction::East, Turn::Left)
            | (Pipe::SouthEast, Direction::South, Turn::Right) => {
                vec![Direction::North, Direction::West]
            }

            (Pipe::SouthWest, Direction::West, Turn::Right)
            | (Pipe::SouthWest, Direction::South, Turn::Left) => {
                vec![Direction::North, Direction::East]
            }
            _ => vec![],
        }
    }
}

struct PipeMap {
    pipes: Vec<Vec<Pipe>>,
    start: Pos2<usize>,
}

impl PipeMap {
    pub fn new(pipes: Vec<Vec<Pipe>>) -> Result<Self, DayError> {
        if pipes.is_empty() || pipes[0].is_empty() {
            return Err(DayError::EmptyMapNoAllowed);
        }
        if !pipes.iter().map(|row| row.len()).all_equal() {
            return Err(DayError::MapMustBerectangle);
        }
        let start = match pipes
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, pipe)| {
                    if matches!(pipe, Pipe::Start) {
                        Some(Pos2::new(x, y))
                    } else {
                        None
                    }
                })
            })
            .exactly_one()
        {
            Ok(pos) => pos,
            Err(_) => return Err(DayError::NeedExactlyOneStart),
        };

        Ok(Self { pipes, start })
    }

    pub fn analyze_loop(&self) -> Result<LoopAnalysis, DayError> {
        for dir in Direction::iter() {
            let mut markings = vec![vec![Mark::Unknown; self.pipes[0].len()]; self.pipes.len()];
            let mut exit = dir;
            let mut turns = HandednessCheck::default();
            let mut pos = self.start;

            for steps in 1.. {
                pos.safe_matrix_set(&mut markings, Mark::Loop);
                let Some((current, pipe)) = pos.safe_matrix_add_and_get(&self.pipes, exit) else {
                    break;
                };
                if matches!(pipe, Pipe::Start) {
                    turns.report_turn(exit, dir)?;
                    return Ok(LoopAnalysis {
                        steps,
                        exit: dir,
                        markings,
                        handedness: turns.get_handedness()?,
                    });
                }
                let Some(next_exit) = pipe.exit(exit.turn_back()) else {
                    break;
                };
                turns.report_turn(exit, next_exit)?;
                exit = next_exit;
                pos = current;
            }
        }
        Err(DayError::NoLoopFound)
    }

    pub fn mark_inside(mark: &mut [Vec<Mark>], start: Pos2<usize>) {
        let mut queue = vec![start];
        while let Some(current) = queue.pop() {
            current.safe_matrix_set(mark, Mark::Inside);
            for dir in Direction::iter() {
                if let Some((next, Mark::Unknown)) = current.safe_matrix_add_and_get(mark, dir) {
                    queue.push(next)
                }
            }
        }
    }

    pub fn count_enclosed(&self) -> Result<usize, DayError> {
        let LoopAnalysis {
            steps: _,
            mut exit,
            handedness,
            mut markings,
        } = self.analyze_loop()?;
        let mut pos = self.start;

        loop {
            let Some((current, pipe)) = pos.safe_matrix_add_and_get(&self.pipes, exit) else {
                // This can actually never happen, we were here before!
                break;
            };
            if matches!(pipe, Pipe::Start) {
                return Ok(markings
                    .into_iter()
                    .flat_map(|row| row.into_iter().filter(|mark| matches!(mark, Mark::Inside)))
                    .count());
            }
            let Some(next_exit) = pipe.exit(exit.turn_back()) else {
                // This can actually never happen, we were here before!
                break;
            };

            for inside in pipe.get_inside_directions(next_exit, handedness) {
                if let Some((mark_pos, Mark::Unknown)) =
                    current.safe_matrix_add_and_get(&markings, inside)
                {
                    Self::mark_inside(&mut markings, mark_pos);
                }
            }
            exit = next_exit;
            pos = current;
        }
        Err(DayError::NoLoopFound)
    }
}

#[derive(Debug, Default)]
struct HandednessCheck {
    left: usize,
    right: usize,
}

impl HandednessCheck {
    pub fn report_turn(&mut self, from: Direction, to: Direction) -> Result<(), DayError> {
        let turn = from.get_turn(to);
        match turn {
            crate::common::turn::Turn::Forward => {}
            crate::common::turn::Turn::Left => self.left += 1,
            crate::common::turn::Turn::Back => return Err(DayError::CantTurnBack),
            crate::common::turn::Turn::Right => self.right += 1,
        }
        Ok(())
    }

    // We get the handedness of the loop because one kind of turns must
    // appear exactly 4 times more often than the other
    pub fn get_handedness(&self) -> Result<Turn, DayError> {
        if self.left + 4 == self.right {
            Ok(Turn::Right)
        } else if self.right + 4 == self.left {
            Ok(Turn::Left)
        } else {
            Err(DayError::IllegalTurns)
        }
    }
}

struct LoopAnalysis {
    steps: usize,
    exit: Direction,
    markings: Vec<Vec<Mark>>,
    handedness: Turn,
}

impl FromStr for PipeMap {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pipes = s
            .lines()
            .map(|line| line.chars().map(|p| p.try_into()).try_collect())
            .try_collect()?;
        Self::new(pipes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Unknown,
    Loop,
    Inside,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::days::{read_string, ResultType, UnitResult};

    #[test]
    fn test_part1() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;
        let expected = ResultType::Integer(8);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example04.txt")?;
        let expected = ResultType::Integer(10);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn example1() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let map: PipeMap = input.parse()?;

        assert_eq!(map.start, Pos2::new(1, 1));
        assert_eq!(map.analyze_loop()?.steps, 8);
        assert_eq!(map.count_enclosed()?, 1);

        Ok(())
    }

    #[test]
    fn example2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;
        let map: PipeMap = input.parse()?;

        assert_eq!(map.start, Pos2::new(0, 2));
        assert_eq!(map.analyze_loop()?.steps, 16);

        Ok(())
    }

    #[test]
    fn example3() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example03.txt")?;
        let map: PipeMap = input.parse()?;

        assert_eq!(map.count_enclosed()?, 8);

        Ok(())
    }

    #[test]
    fn example4() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example04.txt")?;
        let map: PipeMap = input.parse()?;

        assert_eq!(map.count_enclosed()?, 10);

        Ok(())
    }
}
