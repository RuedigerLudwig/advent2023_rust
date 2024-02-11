use super::{DayTrait, DayType, RResult};
use crate::common::{pos2::Pos2, pos3::Pos3};
use itertools::Itertools;
use std::{collections::HashSet, num, str::FromStr};

const DAY_NUMBER: DayType = 22;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let pile: Pile = input.parse()?;
        let settled = SettledPile::create(pile);

        Ok(settled.disintegratable_count().into())
    }

    fn part2(&self, input: &str) -> RResult {
        let pile: Pile = input.parse()?;
        let settled = SettledPile::create(pile);

        Ok(settled.count_falling().into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Illegal brick: {0} ~ {1}")]
    IllegalBrick(Pos3<usize>, Pos3<usize>),
    #[error("Bricks muts not intersect")]
    BricksIntersect,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Brick {
    direction: Direction,
    start: Pos2<usize>,
    z_pos: usize,
    length: usize,
}

impl PartialOrd for Brick {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Brick {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.z_pos.cmp(&other.z_pos) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.start.x().cmp(&other.start.x()) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.start.y().cmp(&other.start.y())
    }
}

impl Brick {
    fn create(start: Pos2<usize>, z_pos: usize, length: usize, direction: Direction) -> Self {
        Self {
            start,
            z_pos,
            length,
            direction,
        }
    }

    pub fn new(first: Pos3<usize>, second: Pos3<usize>) -> Result<Self, DayError> {
        if first.x() != second.x() {
            if first.y() != second.y() || first.z() != second.z() {
                Err(DayError::IllegalBrick(first, second))
            } else if first.x() < second.x() {
                Ok(Self::create(
                    first.project_xy(),
                    first.z(),
                    second.x() - first.x() + 1,
                    Direction::X,
                ))
            } else {
                Ok(Self::create(
                    second.project_xy(),
                    second.z(),
                    first.x() - second.x() + 1,
                    Direction::X,
                ))
            }
        } else if first.y() != second.y() {
            if first.z() != second.z() {
                Err(DayError::IllegalBrick(first, second))
            } else if first.y() < second.y() {
                Ok(Self::create(
                    first.project_xy(),
                    first.z(),
                    second.y() - first.y() + 1,
                    Direction::Y,
                ))
            } else {
                Ok(Self::create(
                    second.project_xy(),
                    second.z(),
                    first.x() - second.x() + 1,
                    Direction::Y,
                ))
            }
        } else if first.z() < second.z() {
            Ok(Self::create(
                first.project_xy(),
                first.z(),
                second.z() - first.z() + 1,
                Direction::Z,
            ))
        } else {
            Ok(Self::create(
                second.project_xy(),
                second.z(),
                first.z() - second.z() + 1,
                Direction::X,
            ))
        }
    }

    fn parse_pos(pos: &str) -> Result<Pos3<usize>, DayError> {
        let parts: Vec<usize> = pos.split(',').map(|s| s.parse()).try_collect()?;
        if parts.len() != 3 {
            return Err(DayError::ParseError(pos.to_owned()));
        }
        Ok(Pos3::new(parts[0], parts[1], parts[2]))
    }

    pub fn contains(&self, block: &Pos3<usize>) -> bool {
        match self.direction {
            Direction::X => {
                block.y() == self.start.y()
                    && block.z() == self.z_pos
                    && (self.start.x()..(self.start.x() + self.length)).contains(&block.x())
            }
            Direction::Y => {
                block.x() == self.start.x()
                    && block.z() == self.z_pos
                    && (self.start.y()..(self.start.y() + self.length)).contains(&block.y())
            }
            Direction::Z => {
                block.x() == self.start.x()
                    && block.y() == self.start.y()
                    && (self.z_pos..(self.z_pos + self.length)).contains(&block.z())
            }
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.blocks().any(|block| other.contains(&block))
    }

    pub fn floor(&self) -> impl Iterator<Item = Pos3<usize>> + '_ {
        BlockIterator::new(self, true)
    }

    pub fn blocks(&self) -> impl Iterator<Item = Pos3<usize>> + '_ {
        BlockIterator::new(self, false)
    }

    fn set_z_pos(&self, z_pos: usize) -> Self {
        Self {
            direction: self.direction,
            start: self.start,
            z_pos,
            length: self.length,
        }
    }
}

struct BlockIterator {
    current: Option<Pos3<usize>>,
    end: Pos3<usize>,
    step: Pos3<usize>,
}

impl BlockIterator {
    pub fn new(brick: &Brick, floor: bool) -> Self {
        match brick.direction {
            Direction::X => Self {
                current: Some(brick.start.expand_z(brick.z_pos)),
                end: Pos3::new(
                    brick.start.x() + (brick.length - 1),
                    brick.start.y(),
                    brick.z_pos,
                ),
                step: Pos3::new(1, 0, 0),
            },
            Direction::Y => Self {
                current: Some(brick.start.expand_z(brick.z_pos)),
                end: Pos3::new(
                    brick.start.x(),
                    brick.start.y() + (brick.length - 1),
                    brick.z_pos,
                ),
                step: Pos3::new(0, 1, 0),
            },
            Direction::Z => {
                if floor {
                    Self {
                        current: Some(brick.start.expand_z(brick.z_pos)),
                        end: brick.start.expand_z(brick.z_pos),
                        step: Pos3::new(0, 0, 1),
                    }
                } else {
                    Self {
                        current: Some(brick.start.expand_z(brick.z_pos)),
                        end: Pos3::new(
                            brick.start.x(),
                            brick.start.y(),
                            brick.z_pos + (brick.length - 1),
                        ),
                        step: Pos3::new(0, 0, 1),
                    }
                }
            }
        }
    }
}

impl Iterator for BlockIterator {
    type Item = Pos3<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        if current == self.end {
            self.current = None;
        } else {
            self.current = Some(current + self.step);
        }
        Some(current)
    }
}

impl FromStr for Brick {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((start, end)) = s.split_once('~') else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let start = Self::parse_pos(start)?;
        let end = Self::parse_pos(end)?;

        Self::new(start, end)
    }
}

struct Pile {
    bricks: Vec<Brick>,
}

impl Pile {
    pub fn new(raw_bricks: Vec<Brick>) -> Result<Self, DayError> {
        let mut bricks: Vec<Brick> = Vec::new();
        for brick in raw_bricks.into_iter().sorted() {
            if bricks.iter().any(|b| b.intersects(&brick)) {
                return Err(DayError::BricksIntersect);
            }
            bricks.push(brick);
        }
        Ok(Self { bricks })
    }
}

impl FromStr for Pile {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.lines().map(|line| line.parse()).try_collect()?)
    }
}

struct SettledBrick {
    brick: Brick,
    foundation: Vec<usize>,
    supported: Vec<usize>,
}
struct SettledPile {
    bricks: Vec<SettledBrick>,
}

impl SettledPile {
    pub fn create(pile: Pile) -> Self {
        let mut bricks: Vec<SettledBrick> = Vec::new();
        for brick in pile.bricks {
            let mut min_z = 0;
            let mut foundation = Vec::new();
            if brick.z_pos > 1 {
                for block in brick.floor() {
                    let mut found = false;
                    for z in (1..brick.z_pos).rev() {
                        if z < min_z {
                            break;
                        }
                        let lower_block = block.set_z(z);
                        for (lower_pos, lower) in bricks.iter().enumerate().rev() {
                            if lower.brick.contains(&lower_block) {
                                if z > min_z {
                                    min_z = z;
                                    foundation.clear();
                                }
                                if !foundation.contains(&lower_pos) {
                                    foundation.push(lower_pos);
                                }
                                found = true;
                            }
                        }
                        if found {
                            break;
                        }
                    }
                }
            }
            let index = bricks.len();
            for idx in foundation.iter() {
                bricks[*idx].supported.push(index);
            }
            let new_brick = SettledBrick {
                brick: brick.set_z_pos(min_z + 1),
                foundation,
                supported: vec![],
            };
            bricks.push(new_brick);
        }
        Self { bricks }
    }

    pub fn disintegratable_count(&self) -> usize {
        self.bricks.len() - self.stabelizers().len()
    }

    fn stabelizers(&self) -> HashSet<usize> {
        self.bricks
            .iter()
            .filter(|b| b.foundation.len() == 1)
            .flat_map(|b| b.foundation.iter().copied())
            .collect()
    }

    pub fn count_falling(&self) -> usize {
        self.stabelizers()
            .into_iter()
            .map(|brick| {
                let mut removed = HashSet::new();
                removed.insert(brick);
                self.check_falling(brick, &mut removed);
                removed.len() - 1
            })
            .sum()
    }

    fn check_falling(&self, brick: usize, missing: &mut HashSet<usize>) {
        for dependend in self.bricks[brick].supported.iter().copied() {
            if self.bricks[dependend]
                .foundation
                .iter()
                .all(|f| missing.contains(f))
            {
                missing.insert(dependend);
            }
        }
        for dependend in self.bricks[brick].supported.iter().copied() {
            if missing.contains(&dependend) {
                self.check_falling(dependend, missing);
            }
        }
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
        let expected = ResultType::Integer(5);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(7);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);
        //155815 too high

        Ok(())
    }

    #[test]
    fn parse_horiz() -> UnitResult {
        let input = "1,0,1~1,2,1";
        let brick: Brick = input.parse()?;
        assert_eq!(brick.direction, Direction::Y);
        assert_eq!(brick.start, Pos2::new(1, 0));
        assert_eq!(brick.z_pos, 1);
        assert_eq!(brick.length, 3);
        assert_eq!(
            brick.floor().collect_vec(),
            [Pos3::new(1, 0, 1), Pos3::new(1, 1, 1), Pos3::new(1, 2, 1)],
        );
        assert_eq!(
            brick.blocks().collect_vec(),
            [Pos3::new(1, 0, 1), Pos3::new(1, 1, 1), Pos3::new(1, 2, 1)],
        );

        Ok(())
    }

    #[test]
    fn parse_vert() -> UnitResult {
        let input = "1,1,9~1,1,8";
        let brick: Brick = input.parse()?;
        assert_eq!(brick.direction, Direction::Z);
        assert_eq!(brick.start, Pos2::new(1, 1));
        assert_eq!(brick.z_pos, 8);
        assert_eq!(brick.length, 2);
        assert_eq!(brick.floor().collect_vec(), [Pos3::new(1, 1, 8)],);
        assert_eq!(
            brick.blocks().collect_vec(),
            [Pos3::new(1, 1, 8), Pos3::new(1, 1, 9)],
        );

        Ok(())
    }

    #[test]
    fn parse_pile() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let pile: Pile = input.parse()?;
        assert_eq!(pile.bricks.len(), 7);

        Ok(())
    }

    #[test]
    fn settle_pile() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let pile: Pile = input.parse()?;
        let settled = SettledPile::create(pile);
        assert_eq!(settled.stabelizers().len(), 2);
        assert_eq!(settled.disintegratable_count(), 5);
        assert_eq!(settled.count_falling(), 7);

        Ok(())
    }
}
