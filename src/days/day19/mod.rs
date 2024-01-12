use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{num, str::FromStr};

const DAY_NUMBER: DayType = 19;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let system: System = input.try_into()?;
        Ok(system.value().into())
    }

    fn part2(&self, input: &str) -> RResult {
        let system: System = input.try_into()?;
        Ok(system
            .count_fitting(PartRange::splat(Range::new(1, 4_000)))
            .into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Unknown Workflow: {0}")]
    UnknownWorkflow(String),
}

#[derive(Debug, PartialEq, Eq)]
enum Progress<'a> {
    Reject,
    Accept,
    Continue(&'a str),
}

impl<'a> From<&'a str> for Progress<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "R" => Progress::Reject,
            "A" => Progress::Accept,
            _ => Progress::Continue(value),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Param {
    X,
    M,
    A,
    S,
}

impl FromStr for Param {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x" => Ok(Param::X),
            "m" => Ok(Param::M),
            "a" => Ok(Param::A),
            "s" => Ok(Param::S),
            _ => Err(DayError::ParseError(s.to_owned())),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Condition {
    GreaterThan(Param, usize),
    LowerThan(Param, usize),
    Always,
}

impl Condition {
    pub fn check(&self, part: &Part) -> bool {
        match self {
            Condition::GreaterThan(param, value) => part.get(param) > *value,
            Condition::LowerThan(param, value) => part.get(param) < *value,
            Condition::Always => true,
        }
    }

    pub fn check_range(&self, range: PartRange) -> (Option<PartRange>, Option<PartRange>) {
        match self {
            Condition::GreaterThan(param, value) => (
                range.set_min(param, value + 1),
                range.set_max(param, *value),
            ),
            Condition::LowerThan(param, value) => (
                range.set_max(param, value - 1),
                range.set_min(param, *value),
            ),
            Condition::Always => (Some(range), None),
        }
    }
}

impl FromStr for Condition {
    type Err = DayError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some((name, value)) = input.split_once('>') {
            Ok(Condition::GreaterThan(name.parse()?, value.parse()?))
        } else if let Some((name, value)) = input.split_once('<') {
            Ok(Condition::LowerThan(name.parse()?, value.parse()?))
        } else {
            Err(DayError::ParseError(input.to_owned()))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Rule<'a> {
    condition: Condition,
    progress: Progress<'a>,
}

impl<'a> TryFrom<&'a str> for Rule<'a> {
    type Error = DayError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Some((condition, progress)) = value.split_once(':') {
            Ok(Rule {
                condition: condition.parse()?,
                progress: progress.into(),
            })
        } else {
            Ok(Rule {
                condition: Condition::Always,
                progress: value.into(),
            })
        }
    }
}

impl Rule<'_> {
    pub fn apply(&self, part: &Part) -> Option<&Progress> {
        if self.condition.check(part) {
            Some(&self.progress)
        } else {
            None
        }
    }

    fn apply_range(&self, range: PartRange) -> (Option<(PartRange, &Progress)>, Option<PartRange>) {
        let (this, next) = self.condition.check_range(range);
        (this.map(|range| (range, &self.progress)), next)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Workflow<'a> {
    name: &'a str,
    rules: Vec<Rule<'a>>,
}

impl<'a> TryFrom<&'a str> for Workflow<'a> {
    type Error = DayError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let Some((name, rest)) = value.split_once('{') else {
            return Err(DayError::ParseError(value.to_owned()));
        };
        let Some(rules) = rest.strip_suffix('}') else {
            return Err(DayError::ParseError(value.to_owned()));
        };
        let rules = rules.split(',').map(Rule::try_from).try_collect()?;
        Ok(Self { name, rules })
    }
}

impl Workflow<'_> {
    pub fn is_accepted(&self, part: &Part) -> &Progress {
        for rule in self.rules.iter() {
            if let Some(progress) = rule.apply(part) {
                return progress;
            }
        }
        unreachable!()
    }
}

struct Workflows<'a> {
    workflows: Vec<Workflow<'a>>,
}
impl<'a> Workflows<'a> {
    fn create<I>(iter: &mut I) -> Result<Self, DayError>
    where
        I: Iterator<Item = &'a str> + Clone,
    {
        let workflows = iter
            .take_while_ref(|line| !line.is_empty())
            .map(|line| line.try_into())
            .try_collect()?;
        Ok(Self { workflows })
    }
}

impl Workflows<'_> {
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.workflows.len()
    }

    pub fn find(&self, name: &str) -> Result<&Workflow<'_>, DayError> {
        self.workflows
            .iter()
            .find(|wf| wf.name == name)
            .ok_or(DayError::UnknownWorkflow(name.to_owned()))
    }

    pub fn is_accepted(&self, part: &Part) -> bool {
        let mut current = "in";
        loop {
            let rule = self.find(current).unwrap();
            match rule.is_accepted(part) {
                Progress::Reject => return false,
                Progress::Accept => return true,
                Progress::Continue(next_rule) => current = *next_rule,
            }
        }
    }

    fn count_by_workflow(&self, mut range: PartRange, name: &str) -> usize {
        let mut count = 0;
        let wf = self.find(name).unwrap();
        for rule in wf.rules.iter() {
            let (this, next) = rule.apply_range(range);
            if let Some((range, progress)) = this {
                match progress {
                    Progress::Reject => {}
                    Progress::Accept => count += range.count(),
                    Progress::Continue(name) => {
                        count += self.count_by_workflow(range, name);
                    }
                }
            }
            if let Some(next_range) = next {
                range = next_range;
            }
        }
        count
    }

    pub fn count_accepted(&self, range: PartRange) -> usize {
        self.count_by_workflow(range, "in")
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Part(usize, usize, usize, usize);

impl Part {
    pub fn value(&self) -> usize {
        self.0 + self.1 + self.2 + self.3
    }

    fn get(&self, param: &Param) -> usize {
        match param {
            Param::X => self.0,
            Param::M => self.1,
            Param::A => self.2,
            Param::S => self.3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Range {
    min: usize,
    max: usize,
}

impl Range {
    pub fn new(min: usize, max: usize) -> Self {
        assert!(min <= max);
        Self { min, max }
    }

    pub fn count(&self) -> usize {
        self.max - self.min + 1
    }

    pub fn set_min(mut self, min: usize) -> Option<Self> {
        self.min = self.min.max(min);
        if self.min > self.max {
            None
        } else {
            Some(self)
        }
    }

    pub fn set_max(mut self, max: usize) -> Option<Self> {
        self.max = self.max.min(max);
        if self.min > self.max {
            None
        } else {
            Some(self)
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PartRange(Range, Range, Range, Range);

impl PartRange {
    pub fn splat(range: Range) -> Self {
        Self(range, range, range, range)
    }

    fn set_max(mut self, param: &Param, value: usize) -> Option<Self> {
        match param {
            Param::X => match self.0.set_max(value) {
                Some(range) => {
                    self.0 = range;
                    Some(self)
                }
                None => None,
            },
            Param::M => match self.1.set_max(value) {
                Some(range) => {
                    self.1 = range;
                    Some(self)
                }
                None => None,
            },
            Param::A => match self.2.set_max(value) {
                Some(range) => {
                    self.2 = range;
                    Some(self)
                }
                None => None,
            },
            Param::S => match self.3.set_max(value) {
                Some(range) => {
                    self.3 = range;
                    Some(self)
                }
                None => None,
            },
        }
    }

    fn set_min(mut self, param: &Param, value: usize) -> Option<Self> {
        match param {
            Param::X => match self.0.set_min(value) {
                Some(range) => {
                    self.0 = range;
                    Some(self)
                }
                None => None,
            },
            Param::M => match self.1.set_min(value) {
                Some(range) => {
                    self.1 = range;
                    Some(self)
                }
                None => None,
            },
            Param::A => match self.2.set_min(value) {
                Some(range) => {
                    self.2 = range;
                    Some(self)
                }
                None => None,
            },
            Param::S => match self.3.set_min(value) {
                Some(range) => {
                    self.3 = range;
                    Some(self)
                }
                None => None,
            },
        }
    }

    fn count(&self) -> usize {
        self.0.count() * self.1.count() * self.2.count() * self.3.count()
    }
}

fn get_pair(s: &str) -> Result<(&str, &str), DayError> {
    s.split_once('=').ok_or(DayError::ParseError(s.to_owned()))
}

impl FromStr for Part {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix('{').and_then(|s| s.strip_suffix('}')) else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        s.split(',')
            .map(get_pair)
            .try_fold(Part::default(), |mut part, split| {
                let (name, value) = split?;
                let value = value.parse()?;
                match name {
                    "x" => part.0 = value,
                    "m" => part.1 = value,
                    "a" => part.2 = value,
                    "s" => part.3 = value,
                    _ => return Err(DayError::ParseError(name.to_owned())),
                }
                Ok(part)
            })
    }
}

struct System<'a> {
    workflows: Workflows<'a>,
    parts: Vec<Part>,
}

impl<'a> TryFrom<&'a str> for System<'a> {
    type Error = DayError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut lines = value.lines();
        let workflows = Workflows::create(&mut lines)?;
        let _ = lines.next();
        let parts = lines.map(|p| p.parse()).try_collect()?;
        Ok(Self { workflows, parts })
    }
}

impl System<'_> {
    pub fn value(&self) -> usize {
        self.parts
            .iter()
            .filter(|part| self.workflows.is_accepted(part))
            .map(|part| part.value())
            .sum()
    }

    pub fn count_fitting(&self, range: PartRange) -> usize {
        self.workflows.count_accepted(range)
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
        let expected = ResultType::Integer(19114);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(167409079868000);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn parse_workflow() -> UnitResult {
        let input = "px{a<2006:qkq,m>2090:A,rfg}";
        let workflow: Workflow = input.try_into()?;
        let expected = Workflow {
            name: "px",
            rules: vec![
                Rule {
                    condition: Condition::LowerThan(Param::A, 2006),
                    progress: Progress::Continue("qkq"),
                },
                Rule {
                    condition: Condition::GreaterThan(Param::M, 2090),
                    progress: Progress::Accept,
                },
                Rule {
                    condition: Condition::Always,
                    progress: Progress::Continue("rfg"),
                },
            ],
        };
        assert_eq!(workflow, expected);

        Ok(())
    }

    #[test]
    fn parse_part() -> UnitResult {
        let input = "{x=787,m=2655,a=1222,s=2876}";
        let part: Part = input.parse()?;
        let expected = Part(787, 2655, 1222, 2876);
        assert_eq!(part, expected);

        Ok(())
    }

    #[test]
    fn parse_all() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let system: System = input.as_str().try_into()?;
        assert_eq!(system.workflows.len(), 11);
        assert_eq!(system.parts.len(), 5);

        Ok(())
    }

    #[test]
    fn is_accepted() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let system: System = input.as_str().try_into()?;

        assert!(system.workflows.is_accepted(&system.parts[0]));
        assert!(!system.workflows.is_accepted(&system.parts[1]));

        Ok(())
    }
}
