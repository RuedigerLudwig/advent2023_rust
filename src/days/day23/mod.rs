use super::{DayTrait, DayType, RResult};
use crate::common::{direction::Direction, pos2::Pos2};
use itertools::Itertools;
use std::{collections::HashMap, str::FromStr};

const DAY_NUMBER: DayType = 23;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let map: ForestMap = input.parse()?;
        Ok(map.go_on_hike()?.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let mut map: ForestMap = input.parse()?;
        map.remove_slopes();
        Ok(map.go_on_hike()?.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("unknown tile: {0}")]
    UnknowTile(char),
    #[error("Map must not be empty")]
    MapMustNotBeEmpty,
    #[error("Map must be a rectangle")]
    MapMustBeRectangle,
    #[error("Map must have exactly one start and finish")]
    MustHaveExactOneStartAndEnd,
    #[error("Map must be surrounded by forest")]
    MustBeSurroundedByForrest,
    #[error("No path found")]
    NoPathFound,
}

#[derive(Debug, Clone, Copy)]
enum Tile {
    Forest,
    Path,
    Slope(Direction),
}

impl TryFrom<char> for Tile {
    type Error = DayError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '#' => Ok(Tile::Forest),
            '.' => Ok(Tile::Path),
            '>' => Ok(Tile::Slope(Direction::East)),
            '^' => Ok(Tile::Slope(Direction::North)),
            '<' => Ok(Tile::Slope(Direction::West)),
            'v' => Ok(Tile::Slope(Direction::South)),
            _ => Err(DayError::UnknowTile(value)),
        }
    }
}

#[derive(Debug, Clone)]
struct Step {
    start: Pos2<usize>,
    direction: Direction,
    reached: Pos2<usize>,
}

impl Step {
    pub fn create(start: Pos2<usize>, direction: Direction, reached: Pos2<usize>) -> Self {
        Self {
            start,
            direction,
            reached,
        }
    }
}

#[derive(Debug)]
enum BranchType {
    DeadEnd(Pos2<usize>),
    Single(Step),
    Branch(Pos2<usize>, Vec<Step>),
}

#[derive(Debug, Clone)]
struct BranchConnection {
    end: Pos2<usize>,
    steps: usize,
}

struct ForestMap {
    start: Pos2<usize>,
    finish: Pos2<usize>,
    map: Vec<Vec<Tile>>,
    slippery_slopes: bool,
}

impl ForestMap {
    pub fn new(map: Vec<Vec<Tile>>) -> Result<Self, DayError> {
        if map.is_empty() || map[0].is_empty() {
            return Err(DayError::MapMustNotBeEmpty);
        }
        if !map.iter().map(|row| row.len()).all_equal() {
            return Err(DayError::MapMustBeRectangle);
        }
        let start = map[0]
            .iter()
            .enumerate()
            .filter(|(_, tile)| matches!(tile, Tile::Path))
            .exactly_one()
            .map_err(|_| DayError::MustHaveExactOneStartAndEnd)
            .map(|(x, _)| Pos2::new(x, 0))?;
        let finish = map[map.len() - 1]
            .iter()
            .enumerate()
            .filter(|(_, tile)| matches!(tile, Tile::Path))
            .exactly_one()
            .map_err(|_| DayError::MustHaveExactOneStartAndEnd)
            .map(|(x, _)| Pos2::new(x, map.len() - 1))?;
        if map.iter().any(|row| {
            !matches!(row[0], Tile::Forest) || !matches!(row[row.len() - 1], Tile::Forest)
        }) {
            return Err(DayError::MustBeSurroundedByForrest);
        }
        if map[0].iter().any(|tile| matches!(tile, Tile::Slope(_)))
            || map[map.len() - 1]
                .iter()
                .any(|tile| matches!(tile, Tile::Slope(_)))
        {
            return Err(DayError::MustBeSurroundedByForrest);
        }
        Ok(Self {
            map,
            start,
            finish,
            slippery_slopes: true,
        })
    }

    fn remove_slopes(&mut self) {
        self.slippery_slopes = false
    }

    fn leave_tile(&self, here: Pos2<usize>) -> Vec<Step> {
        match (self.slippery_slopes, here.safe_matrix_get(&self.map)) {
            (false, Some(Tile::Slope(_))) | (_, Some(Tile::Path)) => Direction::iter()
                .filter_map(|dir| {
                    if let Some((next, tile)) = here.safe_matrix_add_and_get(&self.map, dir)
                        && !matches!(tile, Tile::Forest)
                    {
                        Some(Step::create(here, dir, next))
                    } else {
                        None
                    }
                })
                .collect_vec(),
            (true, Some(Tile::Slope(dir))) => {
                if let Some((next, tile)) = here.safe_matrix_add_and_get(&self.map, *dir) {
                    if !matches!(tile, Tile::Forest) {
                        vec![Step::create(here, *dir, next)]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }

    fn follow_single_trail(&self, prev_step: &Step) -> Result<BranchType, DayError> {
        let start_pos = prev_step.reached;
        let mut possible = self.leave_tile(start_pos);
        match possible.len() {
            0 => Ok(BranchType::DeadEnd(prev_step.reached)),
            1 => {
                let single = possible.pop().unwrap();
                if single.direction == prev_step.direction.turn_back() {
                    Ok(BranchType::DeadEnd(prev_step.reached))
                } else {
                    Ok(BranchType::Single(single))
                }
            }
            2 => {
                possible.retain(|step| step.direction != prev_step.direction.turn_back());
                Ok(BranchType::Single(possible.pop().unwrap()))
            }
            3 | 4 => Ok(BranchType::Branch(start_pos, possible)),
            _ => unreachable!(),
        }
    }

    fn walk_to_next_branch(
        &self,
        prev_step: &Step,
    ) -> Result<Option<(BranchConnection, Vec<Step>)>, DayError> {
        let mut current = prev_step.clone();
        let mut steps = 0;
        loop {
            steps += 1;
            match self.follow_single_trail(&current)? {
                BranchType::Single(step) => {
                    current = step;
                }
                BranchType::DeadEnd(end) => {
                    if end == self.finish {
                        return Ok(Some((BranchConnection { end, steps }, vec![])));
                    } else {
                        return Ok(None);
                    }
                }
                BranchType::Branch(end, possible) => {
                    return Ok(Some((BranchConnection { end, steps }, possible)));
                }
            }
        }
    }

    pub fn go_on_hike(&self) -> Result<usize, DayError> {
        let connections = self.find_paths()?;
        let path = (vec![self.start], 0);
        let mut queue = vec![path];
        let mut max = 0;
        while let Some((path, steps)) = queue.pop() {
            let current = path.last().unwrap();
            if current == &self.finish {
                max = max.max(steps);
                continue;
            }
            let Some(following) = connections.get(current) else {
                continue;
            };
            for branch in following {
                if !path.contains(&branch.end) {
                    let mut new_path = path.clone();
                    new_path.push(branch.end);
                    queue.push((new_path, steps + branch.steps))
                }
            }
        }
        if max == 0 {
            return Err(DayError::NoPathFound);
        };
        Ok(max)
    }

    pub fn find_paths(&self) -> Result<HashMap<Pos2<usize>, Vec<BranchConnection>>, DayError> {
        let Some((first, tile)) = self
            .start
            .safe_matrix_add_and_get(&self.map, Direction::South)
        else {
            return Err(DayError::NoPathFound);
        };
        if matches!(tile, Tile::Forest) {
            return Err(DayError::NoPathFound);
        }
        let step = Step::create(self.start, Direction::South, first);
        let mut queue = vec![step];
        let mut all_connections = HashMap::new();
        let mut seen = vec![];
        while let Some(current) = queue.pop() {
            let Some((connection, next_steps)) = self.walk_to_next_branch(&current)? else {
                continue;
            };
            all_connections
                .entry(current.start)
                .and_modify(|lst: &mut Vec<BranchConnection>| lst.push(connection.clone()))
                .or_insert(vec![connection.clone()]);
            if seen.contains(&connection.end) {
                continue;
            }
            seen.push(connection.end);
            for next_step in next_steps {
                queue.push(next_step)
            }
        }
        Ok(all_connections)
    }
}

impl FromStr for ForestMap {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = s
            .lines()
            .map(|row| row.chars().map(Tile::try_from).try_collect())
            .try_collect()?;
        ForestMap::new(map)
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
        let expected = ResultType::Integer(94);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        // 5990 too low
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(154);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn walk_slippery() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let map: ForestMap = input.parse()?;
        assert_eq!(map.go_on_hike()?, 94);
        Ok(())
    }

    #[test]
    fn walk_not_slippery() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let mut map: ForestMap = input.parse()?;
        map.remove_slopes();
        assert_eq!(map.go_on_hike()?, 154);
        Ok(())
    }
}
