use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{num, str::FromStr};

const DAY_NUMBER: DayType = 4;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let deck: Deck = input.parse()?;
        Ok(deck.winning_values().into())
    }

    fn part2(&self, input: &str) -> RResult {
        let deck: Deck = input.parse()?;
        Ok(deck.collect_winning().into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
}

#[derive(Debug, PartialEq, Eq)]
struct Card {
    id: usize,
    winning: Vec<u32>,
    hand: Vec<u32>,
}

impl FromStr for Card {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((card, numbers)) = s.split_once(':') else {
            return Err(DayError::ParseError(s.to_owned()));
        };

        let Some(id) = card.strip_prefix("Card") else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let id = id.trim().parse()?;

        let Some((winning, hand)) = numbers.split_once('|') else {
            return Err(DayError::ParseError(s.to_owned()));
        };

        let winning = winning
            .split_ascii_whitespace()
            .map(|num| num.parse())
            .try_collect()?;
        let hand = hand
            .split_ascii_whitespace()
            .map(|num| num.parse())
            .try_collect()?;

        Ok(Self { id, winning, hand })
    }
}

impl Card {
    pub fn count_winning_numbers(&self) -> usize {
        self.winning
            .iter()
            .filter(|winning| self.hand.contains(winning))
            .count()
    }

    #[inline]
    pub fn winning_value(&self) -> u32 {
        let winning = self.count_winning_numbers() as u32;
        if winning == 0 {
            0
        } else {
            1 << (winning - 1)
        }
    }
}

struct Deck {
    cards: Vec<Card>,
}

impl FromStr for Deck {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            cards: s.lines().map(|line| line.parse()).try_collect()?,
        })
    }
}

impl Deck {
    pub fn winning_values(&self) -> u32 {
        self.cards.iter().map(|card| card.winning_value()).sum()
    }

    pub fn collect_winning(&self) -> usize {
        self.cards
            .iter()
            .map(|card| card.count_winning_numbers())
            .enumerate()
            .fold(
                (0, vec![1; self.cards.len()]),
                |(sum, mut collected_cards), (idx, winning_numbers)| {
                    let current_card_count = *collected_cards
                        .get(idx)
                        .expect("This is always possible by definition");

                    // At this point I do not care if the winning cards
                    // would exceed the Vec. We take only every up to the
                    // end of the Vec
                    collected_cards[idx + 1..]
                        .iter_mut()
                        .take(winning_numbers)
                        .for_each(|cc| *cc += current_card_count);

                    (sum + current_card_count, collected_cards)
                },
            )
            .0
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
        let expected = ResultType::Integer(13);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(30);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn parse() -> UnitResult {
        let input = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53";
        let card: Card = input.parse()?;
        let expected = Card {
            id: 1,
            winning: vec![41, 48, 83, 86, 17],
            hand: vec![83, 86, 6, 31, 17, 9, 48, 53],
        };
        assert_eq!(card, expected);
        assert_eq!(card.winning_value(), 8);

        Ok(())
    }

    #[test]
    fn collect_winning() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let deck: Deck = input.parse()?;
        assert_eq!(deck.collect_winning(), 30);

        Ok(())
    }
}
