use super::{DayTrait, DayType, RResult};
use crate::common::{direction::Direction, pos2::Pos2};
use colored::Colorize;
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    num,
    str::FromStr,
};

const DAY_NUMBER: DayType = 16;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let mut contraption: Contraption = input.parse()?;
        Ok(contraption.follow_beam().into())
    }

    fn part2(&self, input: &str) -> RResult {
        let mut contraption: Contraption = input.parse()?;
        Ok(contraption.best_all().into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Unknown Mirror: {0}")]
    UnknownMirror(char),
    #[error("Not a ractangle")]
    NotARectangle,
    #[error("Contraption must not be empty")]
    EmptyContraption,
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

enum SplitInfo {
    First(Pos2<usize>, Direction, HashSet<Pos2<usize>>),
    Second(Pos2<usize>, HashSet<Pos2<usize>>),
}

struct Beam {
    pos: Pos2<usize>,
    direction: Direction,
    visited: HashSet<Pos2<usize>>,
    splits: Vec<SplitInfo>,
}

impl Beam {
    pub fn new(start: Pos2<usize>, direction: Direction) -> Self {
        let mut visited = HashSet::new();
        visited.insert(start);
        Self {
            pos: start,
            direction,
            visited,
            splits: vec![],
        }
    }

    pub fn split(&mut self) {
        let mut visited = HashSet::new();
        std::mem::swap(&mut visited, &mut self.visited);
        self.splits
            .push(SplitInfo::First(self.pos, self.direction, visited));
    }

    pub fn walk(&mut self, mirrors: &[Vec<Mirror>]) -> Option<Pos2<usize>> {
        self.visited.insert(self.pos);
        if let Some(mut next_pos) = self.pos.safe_matrix_add(mirrors, self.direction) {
            std::mem::swap(&mut self.pos, &mut next_pos);
            Some(next_pos)
        } else {
            //println!("  out {} {}", self.pos, self.direction);
            None
        }
    }
}

struct Contraption {
    mirrors: Vec<Vec<Mirror>>,
    known_splits: HashMap<Pos2<usize>, HashSet<Pos2<usize>>>,
}

impl Contraption {
    pub fn next_branch(&mut self, beam: &mut Beam) -> bool {
        match beam.splits.pop() {
            Some(SplitInfo::First(pos, dir, mut visited)) => {
                visited.extend(&beam.visited);
                beam.visited = HashSet::new();
                beam.pos = pos;
                beam.direction = dir.turn_back();
                beam.splits.push(SplitInfo::Second(pos, visited));
                //println!("  back: {} {}", beam.pos, beam.direction);
                if beam.walk(&self.mirrors).is_some() {
                    true
                } else {
                    self.next_branch(beam)
                }
            }
            Some(SplitInfo::Second(pos, mut visited)) => {
                visited.extend(&beam.visited);
                self.known_splits.insert(pos, visited.clone());
                beam.visited = visited;
                self.next_branch(beam)
            }
            None => false,
        }
    }

    fn follow(&mut self, beam: &mut Beam) -> usize {
        self.follow_me(beam).0
    }

    fn follow_me(&mut self, beam: &mut Beam) -> (usize, Vec<Vec<Vec<Direction>>>) {
        let mut touched = vec![vec![vec![]; self.mirrors[0].len()]; self.mirrors.len()];
        loop {
            match beam.pos.safe_matrix_get(&self.mirrors).unwrap() {
                Mirror::None => {}
                Mirror::Horizontal => {
                    if beam.direction.is_vertical() {
                        //println!("  - {}", beam.pos);
                        beam.direction = Direction::West;
                        //if let Some(touched) = self.known_splits.get(&beam.pos) {
                        //beam.visited.extend(touched.iter().copied());
                        //self.next_branch(beam);
                        //} else {
                        beam.split();
                        //}
                    }
                }
                Mirror::Vertical => {
                    if beam.direction.is_horizontal() {
                        //println!("  | {}", beam.pos);
                        beam.direction = Direction::North;
                        //if let Some(touched) = self.known_splits.get(&beam.pos) {
                        //beam.visited.extend(touched.iter().copied());
                        //self.next_branch(beam);
                        //} else {
                        beam.split();
                        //}
                    }
                }
                Mirror::UpRight => {
                    //println!("  / {}", beam.pos);
                    if beam.direction.is_horizontal() {
                        beam.direction = beam.direction.turn_left();
                    } else {
                        beam.direction = beam.direction.turn_right();
                    }
                }
                Mirror::UpLeft => {
                    //println!("  \\ {}", beam.pos);
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
                if !self.next_branch(beam) {
                    return (beam.visited.len(), touched);
                }
            } else {
                prev.push(beam.direction);
                beam.pos.safe_matrix_set(&mut touched, prev);
                if beam.walk(&self.mirrors).is_none() && !self.next_branch(beam) {
                    return (beam.visited.len(), touched);
                }
            }
        }
    }

    pub fn follow_beam(&mut self) -> usize {
        let mut beam = Beam::new(Pos2::new(0, 0), Direction::East);
        let (result, _) = self.follow_me(&mut beam);
        result
    }

    pub fn follow_output(&mut self, beam: &mut Beam) -> usize {
        let (result, touched) = self.follow_me(beam);

        for y in 0..self.mirrors.len() {
            for x in 0..self.mirrors[0].len() {
                let t = &touched[y][x];
                let (red, green, blue) = match t.len() {
                    4 => (255, 0, 0),
                    3 => (255, 255, 0),
                    2 => (0, 255, 0),
                    1 => (255, 255, 255),
                    _ => (0, 0, 0),
                };
                let mut out = match self.mirrors[y][x] {
                    Mirror::None => {
                        if t.len() == 1 {
                            match t[0] {
                                Direction::East => ">",
                                Direction::North => "^",
                                Direction::West => "<",
                                Direction::South => "v",
                            }
                        } else {
                            "."
                        }
                    }
                    Mirror::Horizontal => "-",
                    Mirror::Vertical => "|",
                    Mirror::UpLeft => "\\",
                    Mirror::UpRight => "/",
                }
                .truecolor(red, green, blue);
                if beam.visited.contains(&Pos2::new(x, y)) {
                    out = out.bold();
                }
                print!("{}", out);
            }
            println!();
        }

        result
    }

    fn best_all(&mut self) -> usize {
        let height = self.mirrors.len();
        let width = self.mirrors[0].len();
        let north = (0..width)
            .map(|x| {
                self.known_splits.clear();
                let mut beam = Beam::new(Pos2::new(x, 0), Direction::South);
                self.follow(&mut beam)
            })
            .max()
            .unwrap();
        self.known_splits.clear();
        let south = (0..width)
            .map(|x| {
                self.known_splits.clear();
                let mut beam = Beam::new(Pos2::new(x, height - 1), Direction::North);
                self.follow(&mut beam)
            })
            .max()
            .unwrap();
        self.known_splits.clear();
        let west = (0..height)
            .map(|y| {
                self.known_splits.clear();
                let mut beam = Beam::new(Pos2::new(0, y), Direction::East);
                self.follow(&mut beam)
            })
            .max()
            .unwrap();
        let east = (0..height)
            .map(|y| {
                self.known_splits.clear();
                let mut beam = Beam::new(Pos2::new(width - 1, y), Direction::West);
                self.follow(&mut beam)
            })
            .max()
            .unwrap();

        println!("{} {} {} {}", north, south, east, west);

        north.max(south.max(west.max(east)))
    }

    fn new(mirrors: Vec<Vec<Mirror>>) -> Result<Contraption, DayError> {
        if mirrors.is_empty() || mirrors[0].is_empty() {
            Err(DayError::EmptyContraption)
        } else if !mirrors.iter().map(|row| row.len()).all_equal() {
            Err(DayError::NotARectangle)
        } else {
            Ok(Self {
                mirrors,
                known_splits: HashMap::new(),
            })
        }
    }
}

impl FromStr for Contraption {
    type Err = DayError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::new(
            input
                .lines()
                .map(|row| row.chars().map(|c| c.try_into()).try_collect())
                .try_collect()?,
        )
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
        // 7620 too low
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(51);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn south() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "input.txt")?;
        let mut c: Contraption = input.parse()?;

        let mut beam = Beam::new(Pos2::new(c.mirrors[0].len() - 1, 8), Direction::West);
        let result = c.follow_output(&mut beam);
        println!("{}", result);

        Ok(())
    }
}
