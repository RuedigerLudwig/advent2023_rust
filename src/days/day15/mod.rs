use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::num;

const DAY_NUMBER: DayType = 15;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let seq: Sequence = input.into();
        Ok(seq.hash_sum().into())
    }

    fn part2(&self, input: &str) -> RResult {
        let seq: Sequence = input.into();
        Ok(seq.focus_power()?.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Unkown step: {0}")]
    UnknownStep(String),
}

enum Instruction<'a> {
    Add(&'a str, u32),
    Remove(&'a str),
}

struct Step<'a> {
    instruction: &'a str,
}

impl<'a> Step<'a> {
    fn new(instruction: &'a str) -> Self {
        Self { instruction }
    }

    pub fn full_hash(&self) -> usize {
        Self::hash_me(self.instruction)
    }

    pub fn hash_me(data: &str) -> usize {
        data.chars()
            .fold(0, |hash, c| ((hash + (c as usize)) * 17) % 256)
    }

    pub fn as_instruction(&self) -> Result<Instruction, DayError> {
        if let Some((lens, focal)) = self.instruction.split_once('=') {
            Ok(Instruction::Add(lens, focal.parse()?))
        } else if let Some(lens) = self.instruction.strip_suffix('-') {
            Ok(Instruction::Remove(lens))
        } else {
            Err(DayError::UnknownStep(self.instruction.to_owned()))
        }
    }
}

struct Sequence<'a> {
    steps: Vec<Step<'a>>,
}

impl Sequence<'_> {
    pub fn hash_sum(&self) -> usize {
        self.steps.iter().map(|s| s.full_hash()).sum()
    }

    fn as_boxes(&self) -> Result<Vec<Box>, DayError> {
        let mut boxes = vec![Box::new(); 256];
        for step in self.steps.iter() {
            match step.as_instruction()? {
                Instruction::Add(lens, focal) => boxes[Step::hash_me(lens)].add_lens(lens, focal),
                Instruction::Remove(lens) => boxes[Step::hash_me(lens)].remove_lens(lens),
            }
        }
        Ok(boxes)
    }

    pub fn focus_power(&self) -> Result<u32, DayError> {
        Ok(self
            .as_boxes()?
            .into_iter()
            .enumerate()
            .map(|(pos, boxed)| ((pos + 1) as u32) * boxed.focus_power())
            .sum())
    }
}

impl<'a> From<&'a str> for Sequence<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            steps: value.split(',').map(Step::new).collect_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Box<'a> {
    lenses: Vec<(&'a str, u32)>,
}

impl Box<'_> {
    pub fn new() -> Self {
        Self { lenses: vec![] }
    }

    pub fn focus_power(&self) -> u32 {
        self.lenses
            .iter()
            .enumerate()
            .map(|(pos, (_, focal))| ((pos + 1) as u32) * *focal)
            .sum()
    }

    pub fn remove_lens(&mut self, lens: &str) {
        if let Some(pos) = self
            .lenses
            .iter()
            .position(|(old_lens, _)| old_lens == &lens)
        {
            self.lenses.remove(pos);
        }
    }
}

impl<'a> Box<'a> {
    pub fn add_lens(&mut self, lens: &'a str, focal: u32) {
        for (old_lens, old_focal) in self.lenses.iter_mut() {
            if old_lens == &lens {
                *old_focal = focal;
                return;
            }
        }
        self.lenses.push((lens, focal))
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
        let expected = ResultType::Integer(1320);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(145);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn hash() {
        let input = "HASH";
        let step = Step::new(input);
        assert_eq!(step.full_hash(), 52);
    }

    #[test]
    fn boxes() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let seq: Sequence = input.as_str().into();

        let boxes = seq.as_boxes()?;
        assert_eq!(boxes[0].lenses, [("rn", 1), ("cm", 2)]);

        assert_eq!(seq.focus_power()?, 145);

        Ok(())
    }
}
