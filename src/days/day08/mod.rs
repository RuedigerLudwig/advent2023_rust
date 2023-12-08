use std::str::FromStr;

use crate::common::math::lcm;

use super::{DayTrait, DayType, RResult};
use itertools::Itertools;

const DAY_NUMBER: DayType = 8;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let network: Network = input.try_into()?;
        Ok(network.count_human_steps()?.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let network: Network = input.try_into()?;
        Ok(network.count_ghost_steps()?.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
}

const START: &str = "AAA";
const END: &str = "ZZZ";

#[derive(Debug, PartialEq, Eq)]
struct Node {
    name: String,
    left: String,
    right: String,
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl FromStr for Node {
    type Err = DayError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some((name, next)) = value.split_once('=') else {
            return Err(DayError::ParseError(value.to_owned()));
        };
        //reverse char order to make sorting easier
        let name = name.trim().chars().rev().collect();

        let Some((left, right)) = next.split_once(',') else {
            return Err(DayError::ParseError(value.to_owned()));
        };
        let Some(left) = left.trim().strip_prefix('(') else {
            return Err(DayError::ParseError(value.to_owned()));
        };
        //reverse char order to make sorting easier
        let left = left.chars().rev().collect();

        let Some(right) = right.trim().strip_suffix(')') else {
            return Err(DayError::ParseError(value.to_owned()));
        };
        //reverse char order to make sorting easier
        let right = right.chars().rev().collect();

        Ok(Self { name, left, right })
    }
}

struct Network<'a> {
    instructions: &'a str,
    nodes: Vec<Node>,
}

impl<'a> Network<'a> {
    fn find_node(&self, name: &String) -> Option<&Node> {
        let mut min = 0;
        let mut max = self.nodes.len();
        while min < max {
            let middle = (min + max) / 2;
            let node = &self.nodes[middle];
            match &node.name.cmp(name) {
                std::cmp::Ordering::Less => min = middle,
                std::cmp::Ordering::Equal => return Some(node),
                std::cmp::Ordering::Greater => max = middle,
            }
        }
        None
    }

    pub fn count_human_steps(&self) -> Result<usize, DayError> {
        let Some(mut node) = self.find_node(&String::from(START)) else {
            return Err(DayError::NodeNotFound(START.to_owned()));
        };
        for (steps, turn) in self.instructions.chars().cycle().enumerate() {
            let name = if turn == 'L' { &node.left } else { &node.right };
            if name == END {
                return Ok(steps + 1);
            }
            let Some(next_node) = self.find_node(name) else {
                return Err(DayError::NodeNotFound(name.to_owned()));
            };
            node = next_node;
        }
        unreachable!()
    }

    fn walk_one_path(&self, start: &Node) -> Result<usize, DayError> {
        let mut node = start;
        for (steps, turn) in self.instructions.chars().cycle().enumerate() {
            let name = if turn == 'L' { &node.left } else { &node.right };

            //names are revers, so string means originally ending with
            if name.starts_with('Z') {
                return Ok(steps + 1);
            }

            let Some(next_node) = self.find_node(name) else {
                return Err(DayError::NodeNotFound(name.to_owned()));
            };
            node = next_node;
        }
        unreachable!()
    }

    pub fn count_ghost_steps(&self) -> Result<usize, DayError> {
        self.nodes
            .iter()
            .filter(|node| node.name.starts_with('A'))
            .map(|node| self.walk_one_path(node))
            .fold_ok(1, lcm)
    }
}

impl<'a> TryFrom<&'a str> for Network<'a> {
    type Error = DayError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut lines = value.lines();
        let Some(instructions) = lines.next() else {
            return Err(DayError::ParseError(value.to_owned()));
        };

        let _ = lines.next();

        let nodes: Vec<_> = lines.map(|line| line.parse()).try_collect()?;
        let nodes = nodes.into_iter().sorted().collect_vec();

        Ok(Self {
            instructions,
            nodes,
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
        let expected = ResultType::Integer(6);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example03.txt")?;
        let expected = ResultType::Integer(6);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn parse() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;

        let network: Network = input.as_str().try_into()?;
        assert_eq!(network.instructions, "LLR");
        assert_eq!(network.nodes.len(), 3);
        assert_eq!(
            network.nodes[0],
            Node {
                name: String::from("AAA"),
                left: String::from("BBB"),
                right: String::from("BBB")
            }
        );
        assert_eq!(network.count_human_steps()?, 6);

        Ok(())
    }

    #[test]
    fn walk() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;

        let network: Network = input.as_str().try_into()?;
        assert_eq!(network.count_human_steps()?, 2);

        Ok(())
    }

    #[test]
    fn ghost_walk() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example03.txt")?;

        let network: Network = input.as_str().try_into()?;
        assert_eq!(network.count_ghost_steps()?, 6);

        Ok(())
    }
}
