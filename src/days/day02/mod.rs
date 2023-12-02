use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{num, ops::Add, str::FromStr};

const DAY_NUMBER: DayType = 2;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let supposed = Set::new(12, 13, 14);
        let result = input
            .lines()
            .map(|line| line.parse::<Game>())
            .filter_ok(|game| game.is_possible_with(&supposed))
            .map_ok(|game| game.id)
            .fold_ok(0, Add::add)?;
        Ok(result.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let result = input
            .lines()
            .map(|line| line.parse::<Game>())
            .map_ok(|game| game.minimum_required().power())
            .fold_ok(0, Add::add)?;

        Ok(result.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
}

type IntType = u32;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
struct Set {
    red: IntType,
    green: IntType,
    blue: IntType,
}

impl Set {
    #[inline]
    pub fn new(red: IntType, green: IntType, blue: IntType) -> Self {
        Self { red, green, blue }
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.red <= other.red && self.green <= other.green && self.blue <= other.blue
    }

    pub fn power(&self) -> IntType {
        self.red * self.green * self.blue
    }

    pub fn get_minimal_superset(&self, other: &Self) -> Self {
        Self::new(
            self.red.max(other.red),
            self.green.max(other.green),
            self.blue.max(other.blue),
        )
    }

    pub fn add_red(mut self, red: IntType) -> Self {
        self.red += red;
        self
    }

    pub fn add_green(mut self, green: IntType) -> Self {
        self.green += green;
        self
    }

    pub fn add_blue(mut self, blue: IntType) -> Self {
        self.blue += blue;
        self
    }
}

impl FromStr for Set {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(',').try_fold(Set::default(), |set, item| {
            if let Some(amount) = item.trim_end().strip_suffix("red") {
                let added = amount.trim().parse()?;
                Ok(set.add_red(added))
            } else if let Some(amount) = item.trim_end().strip_suffix("green") {
                let added = amount.trim().parse()?;
                Ok(set.add_green(added))
            } else if let Some(amount) = item.trim_end().strip_suffix("blue") {
                let added = amount.trim().parse()?;
                Ok(set.add_blue(added))
            } else {
                Err(DayError::ParseError(item.to_owned()))
            }
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Game {
    id: u32,
    sets: Vec<Set>,
}

impl Game {
    pub fn is_possible_with(&self, compare: &Set) -> bool {
        self.sets.iter().all(|set| set.is_subset_of(compare))
    }

    pub fn minimum_required(&self) -> Set {
        self.sets
            .iter()
            .copied()
            .reduce(|min, next| min.get_minimal_superset(&next))
            .unwrap_or_default()
    }
}

impl FromStr for Game {
    type Err = DayError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let Some((game, sets)) = input.split_once(':') else {
            return Err(DayError::ParseError(input.to_owned()));
        };

        let Some(id) = game.strip_prefix("Game ") else {
            return Err(DayError::ParseError(input.to_owned()));
        };
        let id = id.parse()?;

        let sets = sets.split(';').map(|set| set.parse()).try_collect()?;

        Ok(Self { id, sets })
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
        let expected = ResultType::Integer(8);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(2286);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn parse() -> UnitResult {
        let input = "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red";
        let game: Game = input.parse()?;
        let expected = Game {
            id: 3,
            sets: vec![Set::new(20, 8, 6), Set::new(4, 13, 5), Set::new(1, 5, 0)],
        };
        assert_eq!(game, expected);

        Ok(())
    }

    #[test]
    fn compare() -> UnitResult {
        let compare = Set::new(12, 13, 14);

        let input = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green";
        let game: Game = input.parse()?;
        assert!(game.is_possible_with(&compare));

        let input = "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red";
        let game: Game = input.parse()?;
        assert!(!game.is_possible_with(&compare));

        Ok(())
    }

    #[test]
    fn minimum() -> UnitResult {
        let input = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green";
        let game: Game = input.parse()?;
        assert_eq!(game.minimum_required(), Set::new(4, 2, 6));

        let input = "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red";
        let game: Game = input.parse()?;
        assert_eq!(game.minimum_required(), Set::new(20, 13, 6));

        Ok(())
    }
}
