use crate::common::{direction::Direction, pos2::Pos2, turn::Turn};

use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{num, str::FromStr};

const DAY_NUMBER: DayType = 18;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let lagoon: Lagoon = input.parse()?;
        let steps = lagoon.walk_path(false).0;
        Ok(steps.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let lagoon: Lagoon = input.parse()?;
        lagoon.print_small(true, 16 * 16 * 16 * 16);
        //let steps = lagoon.walk_path(true).0;
        Ok(5.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
}

struct Instruction {
    turn: Direction,
    steps: i64,
    real_turn: Direction,
    real_steps: i64,
}

impl Instruction {
    pub fn new(turn: Direction, steps: i64, color: &str) -> Self {
        let real_turn = match color.chars().last().unwrap() {
            '0' => Direction::East,
            '1' => Direction::South,
            '2' => Direction::West,
            '3' => Direction::North,
            _ => panic!("Unknow real turn"),
        };
        let real_steps = color
            .chars()
            .take(5)
            .fold(0, |s, c| s * 16 + c.to_digit(16).unwrap() as i64);
        Self {
            turn,
            steps,
            real_steps,
            real_turn,
        }
    }

    #[inline]
    pub fn turn(&self, real: bool) -> Direction {
        if real {
            self.real_turn
        } else {
            self.turn
        }
    }

    #[inline]
    pub fn steps(&self, real: bool) -> i64 {
        if real {
            self.real_steps
        } else {
            self.steps
        }
    }
}

impl FromStr for Instruction {
    type Err = DayError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts = input.split_ascii_whitespace();
        let Some(turn) = parts.next() else {
            return Err(DayError::ParseError(input.to_owned()));
        };
        let turn = match turn {
            "U" => Direction::North,
            "R" => Direction::East,
            "D" => Direction::South,
            "L" => Direction::West,
            _ => return Err(DayError::ParseError(input.to_owned())),
        };
        let Some(steps) = parts.next() else {
            return Err(DayError::ParseError(input.to_owned()));
        };
        let steps = steps.parse()?;
        let Some(color) = parts.next() else {
            return Err(DayError::ParseError(input.to_owned()));
        };
        Ok(Self::new(turn, steps, &color[2..=7]))
    }
}

struct Lagoon {
    instructions: Vec<Instruction>,
}

impl Lagoon {
    pub fn mark_inside(mark: &mut [Vec<bool>], start: Pos2<usize>) {
        let mut queue = vec![start];
        while let Some(current) = queue.pop() {
            current.safe_matrix_set(mark, true);
            for dir in Direction::iter() {
                if let Some((next, false)) = current.safe_matrix_add_and_get(mark, dir) {
                    queue.push(next)
                }
            }
        }
    }

    fn print_small(&self, real: bool, factor: i64) {
        let edges = self
            .instructions
            .iter()
            .scan(Pos2::new(0, 0), |pos, instruction| {
                let steps = instruction.steps(real) / factor;
                match instruction.turn(real) {
                    Direction::East => *pos += Pos2::new(steps, 0),
                    Direction::North => *pos += Pos2::new(0, -steps),
                    Direction::West => *pos += Pos2::new(-steps, 0),
                    Direction::South => *pos += Pos2::new(0, steps),
                }
                Some(*pos)
            })
            .collect_vec();
        let (min, max) = edges
            .iter()
            .fold((Pos2::new(0, 0), Pos2::new(0, 0)), |(min, max), edge| {
                (min.min_components(*edge), max.max_components(*edge))
            });
        let start = Pos2::new(-min.x() as usize, -min.y() as usize);
        let holes =
            vec![vec![false; (max.x() - min.x() + 1) as usize]; (max.y() - min.y() + 1) as usize];
        let (holes, _) =
            self.instructions
                .iter()
                .fold((holes, start), |(mut holes, mut pos), instruction| {
                    let steps = instruction.steps(real) / factor;
                    for _ in 0..steps {
                        pos = pos.safe_matrix_add(&holes, instruction.turn(real)).unwrap();
                        pos.safe_matrix_set(&mut holes, true);
                    }
                    (holes, pos)
                });
        for row in holes {
            for hole in row {
                print!("{}", if hole { '#' } else { '.' })
            }
            println!();
        }
    }

    pub fn walk_path(&self, real: bool) -> (usize, Turn) {
        let (left, right, _) =
            self.instructions
                .iter()
                .fold((0, 0, None), |(left, right, prev), instruction| {
                    if let Some(prev) = prev {
                        match instruction.turn(real).get_turn(prev) {
                            Turn::Back | Turn::Forward => unreachable!(),
                            Turn::Left => (left + 1, right, Some(instruction.turn(real))),
                            Turn::Right => (left, right + 1, Some(instruction.turn(real))),
                        }
                    } else {
                        (left, right, Some(instruction.turn(real)))
                    }
                });
        let edges = self
            .instructions
            .iter()
            .scan(Pos2::new(0, 0), |pos, instruction| {
                match instruction.turn(real) {
                    Direction::East => *pos += Pos2::new(instruction.steps(real), 0),
                    Direction::North => *pos += Pos2::new(0, -instruction.steps(real)),
                    Direction::West => *pos += Pos2::new(-instruction.steps(real), 0),
                    Direction::South => *pos += Pos2::new(0, instruction.steps(real)),
                }
                Some(*pos)
            })
            .collect_vec();
        let (min, max) = edges
            .iter()
            .fold((Pos2::new(0, 0), Pos2::new(0, 0)), |(min, max), edge| {
                (min.min_components(*edge), max.max_components(*edge))
            });
        let start = Pos2::new(-min.x() as usize, -min.y() as usize);
        let holes =
            vec![vec![false; (max.x() - min.x() + 1) as usize]; (max.y() - min.y() + 1) as usize];
        let (holes, _) =
            self.instructions
                .iter()
                .fold((holes, start), |(mut holes, mut pos), instruction| {
                    for _ in 0..instruction.steps(real) {
                        pos = pos.safe_matrix_add(&holes, instruction.turn(real)).unwrap();
                        pos.safe_matrix_set(&mut holes, true);
                    }
                    (holes, pos)
                });
        let turn = if left == right + 4 {
            Turn::Left
        } else {
            Turn::Right
        };

        let (holes, _) =
            self.instructions
                .iter()
                .fold((holes, start), |(mut holes, mut pos), instruction| {
                    let perp = instruction.turn(real) + turn;
                    for _ in 0..instruction.steps(real) - 1 {
                        pos = pos.safe_matrix_add(&holes, instruction.turn(real)).unwrap();
                        let inside = pos.safe_matrix_add(&holes, perp).unwrap();
                        Self::mark_inside(&mut holes, inside);
                    }
                    pos = pos.safe_matrix_add(&holes, instruction.turn(real)).unwrap();
                    (holes, pos)
                });

        (
            holes
                .iter()
                .flat_map(|row| row.iter().filter(|h| **h))
                .count(),
            turn,
        )
    }

    fn extract_pool(&self, real: bool) -> i64 {
        let (turns, _) = self
            .instructions
            .iter()
            .fold((0, None), |(turns, prev), instruction| {
                if let Some(prev) = prev {
                    match instruction.turn(real).get_turn(prev) {
                        Turn::Back | Turn::Forward => unreachable!(),
                        Turn::Left => (turns - 1, Some(instruction.turn(real))),
                        Turn::Right => (turns + 1, Some(instruction.turn(real))),
                    }
                } else {
                    (turns, Some(instruction.turn(real)))
                }
            });
        let turn = if turns == -4 { Turn::Left } else { Turn::Right };
        let (_, area) = self
            .instructions
            .iter()
            .scan(Pos2::new(0, 0), |pos, instruction| {
                let steps = instruction.steps(real);
                let prev_pos = *pos;
                match instruction.turn(real) {
                    Direction::East => {
                        *pos += Pos2::new(steps, 0);
                        Some(Some(Line::new(prev_pos, *pos)))
                    }
                    Direction::North => {
                        *pos += Pos2::new(0, -steps);
                        Some(None)
                    }
                    Direction::West => {
                        *pos += Pos2::new(-steps, 0);
                        Some(Some(Line::new(prev_pos, *pos)))
                    }
                    Direction::South => {
                        *pos += Pos2::new(0, steps);
                        Some(None)
                    }
                }
            })
            .flatten()
            .sorted_by_key(|line| line.from.y())
            .fold((Lines::new(), 0), |(mut lines, mut area), line| {
                if let Some(prev_y) = lines.y {
                    let curr_y = line.from.y();
                    let height = curr_y - prev_y;
                    for prev in lines.lines.iter() {
                        area += (prev.width()) * height;
                    }
                }
                //area += line.width().abs();
                lines.add(line);
                (lines, area)
            });

        area
    }
}

struct Line {
    from: Pos2<i64>,
    to: Pos2<i64>,
}

impl Line {
    fn new(from: Pos2<i64>, to: Pos2<i64>) -> Self {
        Self { from, to }
    }

    fn width(&self) -> i64 {
        if self.to.x() > self.from.x() {
            self.to.x() - self.from.x() + 1
        } else {
            self.to.x() - self.from.x() - 1
        }
    }
}

struct Lines {
    y: Option<i64>,
    lines: Vec<Line>,
}
impl Lines {
    fn new() -> Self {
        Self {
            y: None,
            lines: vec![],
        }
    }

    fn add(&mut self, line: Line) {
        self.y = Some(line.from.y());
        self.lines.push(line);
    }
}

impl FromStr for Lagoon {
    type Err = DayError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            instructions: input.lines().map(|line| line.parse()).try_collect()?,
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
        let expected = ResultType::Integer(62);
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

    #[test]
    fn parse() -> UnitResult {
        let input = "R 6 (#70c710)";
        let instruction: Instruction = input.parse()?;

        assert_eq!(instruction.turn, Direction::East);
        assert_eq!(instruction.steps, 6);
        assert_eq!(instruction.turn(true), Direction::East);
        assert_eq!(instruction.steps(true), 461937);

        Ok(())
    }

    #[test]
    fn walk() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let lagoon: Lagoon = input.parse()?;

        lagoon.print_small(false, 1);
        assert_eq!(lagoon.extract_pool(false), 62);
        assert_eq!(lagoon.extract_pool(true), 952408144115);

        Ok(())
    }
}
