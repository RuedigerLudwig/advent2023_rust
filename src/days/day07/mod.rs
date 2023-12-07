use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{num, str::FromStr};

const DAY_NUMBER: DayType = 7;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let game: Game<RegularCard> = input.parse()?;
        Ok(game.winnings().into())
    }

    fn part2(&self, input: &str) -> RResult {
        let game: Game<BetterCard> = input.parse()?;
        Ok(game.winnings().into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Not a card")]
    NotACard(char),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

trait Card: Ord + Sized {
    fn from_char(ch: char) -> Result<Self, DayError>;
    fn hand_type(hand: &[Self]) -> HandType;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RegularCard(u32);

impl Card for RegularCard {
    fn from_char(ch: char) -> Result<Self, DayError> {
        match ch {
            '2'..='9' => Ok(Self(ch.to_digit(10).unwrap())),
            'T' => Ok(Self(10)),
            'J' => Ok(Self(11)),
            'Q' => Ok(Self(12)),
            'K' => Ok(Self(13)),
            'A' => Ok(Self(14)),
            _ => Err(DayError::NotACard(ch)),
        }
    }

    fn hand_type(hand: &[Self]) -> HandType {
        let num_cards = hand.iter().fold(vec![0; 13], |mut num_cards, card| {
            num_cards[(card.0 as usize) - 2] += 1;
            num_cards
        });
        let max_count = num_cards.iter().max().copied().unwrap();
        match max_count {
            1 => HandType::HighCard,
            2 => {
                if num_cards.iter().filter(|count| **count == 2).count() == 2 {
                    HandType::TwoPair
                } else {
                    HandType::OnePair
                }
            }
            3 => {
                if num_cards.contains(&2) {
                    HandType::FullHouse
                } else {
                    HandType::ThreeOfAKind
                }
            }
            4 => HandType::FourOfAKind,
            5 => HandType::FiveOfAKind,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BetterCard(u32);

impl BetterCard {
    #[inline]
    fn is_joker(&self) -> bool {
        matches!(self, BetterCard(1))
    }
}

impl Card for BetterCard {
    fn from_char(ch: char) -> Result<Self, DayError> {
        match ch {
            '2'..='9' => Ok(Self(ch.to_digit(10).unwrap())),
            'T' => Ok(Self(10)),
            'J' => Ok(Self(1)),
            'Q' => Ok(Self(11)),
            'K' => Ok(Self(12)),
            'A' => Ok(Self(13)),
            _ => Err(DayError::NotACard(ch)),
        }
    }

    fn hand_type(hand: &[Self]) -> HandType {
        let num_cards = hand.iter().fold(vec![0; 13], |mut num_cards, card| {
            num_cards[(card.0 as usize) - 1] += 1;
            num_cards
        });
        let max_count = num_cards.iter().max().unwrap();
        let joker_count = hand.iter().filter(|c| c.is_joker()).count();
        match max_count {
            1 => {
                assert!(joker_count < 2);
                if joker_count == 1 {
                    HandType::OnePair
                } else {
                    HandType::HighCard
                }
            }
            2 => {
                let pair_count = num_cards.iter().filter(|count| **count == 2).count();
                match joker_count {
                    3..=5 => unreachable!(),

                    2 => {
                        if pair_count == 2 {
                            HandType::FourOfAKind
                        } else {
                            HandType::ThreeOfAKind
                        }
                    }
                    1 => {
                        if pair_count == 2 {
                            HandType::FullHouse
                        } else {
                            HandType::ThreeOfAKind
                        }
                    }
                    _ => {
                        if pair_count == 2 {
                            HandType::TwoPair
                        } else {
                            HandType::OnePair
                        }
                    }
                }
            }
            3 => match joker_count {
                3 => {
                    let pair_count = num_cards.iter().filter(|count| **count == 2).count();
                    if pair_count == 1 {
                        HandType::FiveOfAKind
                    } else {
                        HandType::FourOfAKind
                    }
                }
                2 => HandType::FiveOfAKind,
                1 => HandType::FourOfAKind,
                _ => {
                    if num_cards.contains(&2) {
                        HandType::FullHouse
                    } else {
                        HandType::ThreeOfAKind
                    }
                }
            },
            4 => {
                if joker_count == 1 || joker_count == 4 {
                    HandType::FiveOfAKind
                } else {
                    HandType::FourOfAKind
                }
            }
            5 => HandType::FiveOfAKind,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct Hand<C: Card> {
    cards: Vec<C>,
    value: u64,
}

impl<C: Card> Eq for Hand<C> {}
impl<C: Card> PartialEq for Hand<C> {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.cmp(other), std::cmp::Ordering::Equal)
    }
}

impl<C: Card> PartialOrd for Hand<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Card> Ord for Hand<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.get_type().cmp(&other.get_type()) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.cards.cmp(&other.cards)
    }
}

impl<C: Card> Hand<C> {
    pub fn get_type(&self) -> HandType {
        C::hand_type(&self.cards)
    }
}

struct Game<C: Card> {
    hands: Vec<Hand<C>>,
}

impl<C: Card> Game<C> {
    pub fn winnings(&self) -> u64 {
        self.hands
            .iter()
            .sorted()
            .enumerate()
            .map(|(pos, hand)| (pos as u64 + 1) * hand.value)
            .sum()
    }
}

impl<C: Card> FromStr for Game<C> {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hands = s.lines().map(|line| line.parse()).try_collect()?;
        Ok(Self { hands })
    }
}

impl<C: Card> FromStr for Hand<C> {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((hand, value)) = s.split_once(' ') else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let cards: Vec<_> = hand.chars().map(C::from_char).try_collect()?;
        if cards.len() != 5 {
            return Err(DayError::ParseError(s.to_owned()));
        }
        Ok(Self {
            cards,
            value: value.parse()?,
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
        let expected = ResultType::Integer(6440);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(5905);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn parse_type() -> UnitResult {
        let input = "32T3K 765";
        let hand: Hand<RegularCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::OnePair);

        let input = "T55J5 684";
        let hand: Hand<RegularCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::ThreeOfAKind);

        let input = "KTJJT 220";
        let hand: Hand<RegularCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::TwoPair);

        Ok(())
    }

    #[test]
    fn parse_better_type() -> UnitResult {
        let input = "32T4K 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::HighCard);

        let input = "32T3K 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::OnePair);

        let input = "32324 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::TwoPair);

        let input = "34323 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::ThreeOfAKind);

        let input = "33323 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::FourOfAKind);

        let input = "33333 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::FiveOfAKind);

        let input = "32T4J 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::OnePair);

        let input = "J2TJK 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::ThreeOfAKind);

        let input = "32T3J 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::ThreeOfAKind);

        let input = "3232J 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::FullHouse);

        let input = "3J3J4 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::FourOfAKind);

        let input = "3J323 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::FourOfAKind);

        let input = "J4J2J 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::FourOfAKind);

        let input = "3J3J3 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::FiveOfAKind);

        let input = "JJ3J3 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::FiveOfAKind);

        let input = "333J3 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::FiveOfAKind);

        let input = "JJJ2J 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::FiveOfAKind);

        let input = "JJJJJ 1";
        let hand: Hand<BetterCard> = input.parse()?;
        assert_eq!(hand.get_type(), HandType::FiveOfAKind);

        Ok(())
    }
}
