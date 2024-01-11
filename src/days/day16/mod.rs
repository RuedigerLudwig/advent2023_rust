use super::{DayTrait, DayType, RResult};
use crate::common::{direction::Direction, pos2::Pos2};
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
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
        let contraption: Contraption = input.parse()?;
        Ok(contraption
            .single_beam(Pos2::new(0, 0), Direction::East)
            .into())
    }

    fn part2(&self, input: &str) -> RResult {
        let contraption: Contraption = input.parse()?;
        let best_all = contraption.best_all();
        Ok(best_all.into())
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

struct MirrorPath {
    end_points: Vec<Pos2<usize>>,
    energized: HashSet<Pos2<usize>>,
}

struct Contraption {
    mirrors: Vec<Vec<Mirror>>,
    known_splits: HashMap<Pos2<usize>, MirrorPath>,
}

fn first_key<A, B>(set: HashSet<(A, B)>) -> HashSet<A>
where
    A: Eq + Hash,
{
    set.into_iter().map(|(a, _)| a).collect()
}

impl Contraption {
    fn new(mirrors: Vec<Vec<Mirror>>) -> Result<Contraption, DayError> {
        if mirrors.is_empty() || mirrors[0].is_empty() {
            Err(DayError::EmptyContraption)
        } else if !mirrors.iter().map(|row| row.len()).all_equal() {
            Err(DayError::NotARectangle)
        } else {
            let mut contraption = Self {
                mirrors,
                known_splits: HashMap::new(),
            };
            contraption.follow_mirrors();
            Ok(contraption)
        }
    }

    fn single_beam(&self, start: Pos2<usize>, direction: Direction) -> usize {
        let (pos, mut energized) = self.follow_beam(start, direction);
        let Some(pos) = pos else {
            return energized.len();
        };
        let mut seen = vec![];
        let mut queue = vec![pos];
        while let Some(pos) = queue.pop() {
            if seen.contains(&pos) {
                continue;
            }
            seen.push(pos);
            let info = self.known_splits.get(&pos).unwrap();
            energized.extend(&info.energized);
            queue.extend(&info.end_points);
        }
        energized.len()
    }

    fn best_all(&self) -> usize {
        let height = self.mirrors.len();
        let width = self.mirrors[0].len();
        let north = (0..width)
            .map(|x| self.single_beam(Pos2::new(x, 0), Direction::South))
            .max()
            .unwrap();
        let south = (0..width)
            .map(|x| self.single_beam(Pos2::new(x, height - 1), Direction::North))
            .max()
            .unwrap();
        let west = (0..height)
            .map(|y| self.single_beam(Pos2::new(0, y), Direction::East))
            .max()
            .unwrap();
        let east = (0..height)
            .map(|y| self.single_beam(Pos2::new(width - 1, y), Direction::West))
            .max()
            .unwrap();

        vec![north, south, west, east].into_iter().max().unwrap()
    }

    fn follow_mirrors(&mut self) {
        for (y, row) in self.mirrors.iter().enumerate() {
            for (x, mirror) in row.iter().enumerate() {
                match mirror {
                    Mirror::None | Mirror::UpRight | Mirror::UpLeft => {}
                    Mirror::Horizontal => {
                        let pos = Pos2::new(x, y);
                        let (out_east, mut energized) = self.follow_beam(pos, Direction::East);
                        let (out_west, energized_west) = self.follow_beam(pos, Direction::West);
                        let mut end_points = vec![];
                        if let Some(east) = out_east {
                            end_points.push(east);
                        }
                        if let Some(west) = out_west {
                            end_points.push(west);
                        }
                        energized.extend(energized_west);
                        self.known_splits.insert(
                            pos,
                            MirrorPath {
                                end_points,
                                energized,
                            },
                        );
                    }
                    Mirror::Vertical => {
                        let pos = Pos2::new(x, y);
                        let (out_north, mut energized) = self.follow_beam(pos, Direction::North);
                        let (out_south, energized_south) = self.follow_beam(pos, Direction::South);
                        let mut end_points = vec![];
                        if let Some(north) = out_north {
                            end_points.push(north);
                        }
                        if let Some(south) = out_south {
                            end_points.push(south);
                        }
                        energized.extend(energized_south);
                        self.known_splits.insert(
                            pos,
                            MirrorPath {
                                end_points,
                                energized,
                            },
                        );
                    }
                }
            }
        }
    }

    fn follow_beam(
        &self,
        mut pos: Pos2<usize>,
        mut direction: Direction,
    ) -> (Option<Pos2<usize>>, HashSet<Pos2<usize>>) {
        let mut touched = HashSet::new();
        let mut mirror = pos.safe_matrix_get(&self.mirrors).unwrap();
        loop {
            match mirror {
                Mirror::None => {}
                Mirror::Horizontal => {
                    if direction.is_vertical() {
                        return (Some(pos), first_key(touched));
                    }
                }
                Mirror::Vertical => {
                    if direction.is_horizontal() {
                        return (Some(pos), first_key(touched));
                    }
                }
                Mirror::UpRight => {
                    if direction.is_horizontal() {
                        direction = direction.turn_left();
                    } else {
                        direction = direction.turn_right();
                    }
                }
                Mirror::UpLeft => {
                    if direction.is_vertical() {
                        direction = direction.turn_left();
                    } else {
                        direction = direction.turn_right();
                    }
                }
            }

            touched.insert((pos, direction));
            if let Some((next_pos, next_mirror)) =
                pos.safe_matrix_add_and_get(&self.mirrors, direction)
            {
                if touched.contains(&(next_pos, direction)) {
                    return (None, first_key(touched));
                }
                pos = next_pos;
                mirror = next_mirror;
            } else {
                return (None, first_key(touched));
            }
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
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(51);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }
}
