use super::{DayTrait, DayType, RResult};
use crate::common::math::lcm;
use itertools::Itertools;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    fmt::Display,
    num,
};

const DAY_NUMBER: DayType = 20;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let mut config: Configuration = input.try_into()?;
        let (low, high) = config.calc_pulses(1_000);
        Ok((low * high).into())
    }

    fn part2(&self, input: &str) -> RResult {
        let config: Configuration = input.try_into()?;
        let pushes = ComplexSolver::solve(config)?;
        Ok(pushes.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Destinations must not be empty for {0}")]
    DestinationsMustNotBeEmpty(String),
    #[error("No broadcaster found")]
    NoBroadcaster,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Pulse {
    High,
    Low,
}

impl Display for Pulse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pulse::High => write!(f, "high"),
            Pulse::Low => write!(f, "low"),
        }
    }
}

const BUTTON: &str = "button";
const BROADCASTER: &str = "broadcaster";

#[derive(Debug, Clone)]
struct Relay<'a> {
    name: &'a str,
    destinations: Vec<&'a str>,
}

impl Display for Relay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Relay<'_> {
    fn name(&self) -> &str {
        self.name
    }

    fn get_destinations(&self) -> &[&str] {
        &self.destinations
    }
}

#[derive(Debug, Clone)]
struct FlipFlop<'a> {
    name: &'a str,
    is_on: Cell<bool>,
    destinations: Vec<&'a str>,
}

impl FlipFlop<'_> {
    fn name(&self) -> &str {
        self.name
    }

    fn send_pulse(&self) -> Pulse {
        self.is_on.set(!self.is_on.get());
        if self.is_on.get() {
            Pulse::High
        } else {
            Pulse::Low
        }
    }

    fn is_at_start_state(&self) -> bool {
        !self.is_on.get()
    }

    fn get_destinations(&self) -> &[&str] {
        &self.destinations
    }
}

#[derive(Debug, Clone)]
struct Conjunction<'a> {
    name: &'a str,
    prev: RefCell<HashMap<String, Pulse>>,
    destinations: Vec<&'a str>,
}

impl Conjunction<'_> {
    fn name(&self) -> &str {
        self.name
    }

    fn handle_pulse(&self, source: &str, pulse: Pulse) -> Pulse {
        let mut prev = self.prev.borrow_mut();
        prev.insert(source.to_owned(), pulse);
        if prev.values().all(|p| matches!(p, Pulse::High)) {
            Pulse::Low
        } else {
            Pulse::High
        }
    }

    fn announce_source(&self, source: &str) {
        let mut prev = self.prev.borrow_mut();
        prev.insert(source.to_string(), Pulse::Low);
    }

    fn get_destinations(&self) -> &[&str] {
        &self.destinations
    }

    fn is_at_start_state(&self) -> bool {
        let prev = self.prev.borrow();
        prev.values().all(|p| matches!(p, Pulse::Low))
    }
}

#[derive(Debug, Clone)]
enum Module<'a> {
    Relay(Relay<'a>),
    FlipFlop(FlipFlop<'a>),
    Conjunction(Conjunction<'a>),
}

impl Display for Module<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Module::Relay(m) => write!(f, "+{}", m.name()),
            Module::FlipFlop(m) => write!(f, "%{}", m.name()),
            Module::Conjunction(m) => write!(f, "&{}", m.name()),
        }
    }
}

impl<'a> Module<'a> {
    pub fn relay(name: &'a str, destinations: Vec<&'a str>) -> Result<Self, DayError> {
        if destinations.is_empty() {
            Err(DayError::DestinationsMustNotBeEmpty(name.to_string()))
        } else {
            Ok(Self::Relay(Relay { name, destinations }))
        }
    }

    pub fn flipflop(name: &'a str, destinations: Vec<&'a str>) -> Result<Self, DayError> {
        if destinations.is_empty() {
            Err(DayError::DestinationsMustNotBeEmpty(name.to_string()))
        } else {
            Ok(Self::FlipFlop(FlipFlop {
                name,
                is_on: Cell::new(false),
                destinations,
            }))
        }
    }

    pub fn conjunction(name: &'a str, destinations: Vec<&'a str>) -> Result<Self, DayError> {
        if destinations.is_empty() {
            Err(DayError::DestinationsMustNotBeEmpty(name.to_string()))
        } else {
            Ok(Self::Conjunction(Conjunction {
                name,
                prev: RefCell::new(HashMap::new()),
                destinations,
            }))
        }
    }

    fn is_flipflop(&self) -> bool {
        matches!(self, Module::FlipFlop(_))
    }
}

impl<'a> Module<'a> {
    #[inline]
    fn name(&'a self) -> &'a str {
        match self {
            Module::Relay(m) => m.name(),
            Module::FlipFlop(m) => m.name(),
            Module::Conjunction(m) => m.name(),
        }
    }

    #[inline]
    fn announce_source(&self, source: &str) {
        match self {
            Module::Relay(_) | Module::FlipFlop(_) => {}
            Module::Conjunction(m) => m.announce_source(source),
        }
    }

    #[inline]
    fn will_work_on_pulse(&self, pulse: Pulse) -> bool {
        match self {
            Module::Relay(_) | Module::Conjunction(_) => true,
            Module::FlipFlop(_) => matches!(pulse, Pulse::Low),
        }
    }

    #[inline]
    fn handle_pulse(&self, source: &str, pulse: Pulse) -> Pulse {
        match self {
            Module::Relay(_) => pulse,
            Module::FlipFlop(m) => m.send_pulse(),
            Module::Conjunction(m) => m.handle_pulse(source, pulse),
        }
    }

    #[inline]
    fn is_at_start_state(&self) -> bool {
        match self {
            Module::Relay(_) => true,
            Module::FlipFlop(m) => m.is_at_start_state(),
            Module::Conjunction(m) => m.is_at_start_state(),
        }
    }
}

impl<'a> Module<'a> {
    #[inline]
    fn get_destinations(&'a self) -> &'a [&'a str] {
        match self {
            Module::Relay(m) => m.get_destinations(),
            Module::FlipFlop(m) => m.get_destinations(),
            Module::Conjunction(m) => m.get_destinations(),
        }
    }
}

struct Configuration<'a> {
    modules: Vec<Module<'a>>,
}

impl<'a> Configuration<'a> {
    fn create_button() -> Result<Module<'a>, DayError> {
        Module::relay(BUTTON, vec![BROADCASTER])
    }

    fn create_module(value: &'a str) -> Result<Module<'a>, DayError> {
        let Some((module_name, destinations)) = value.split_once("->") else {
            return Err(DayError::ParseError(value.to_owned()));
        };
        let destinations = destinations.split(',').map(|c| c.trim()).collect_vec();

        let module_name = module_name.trim();
        if module_name == BROADCASTER {
            Module::relay(module_name, destinations)
        } else if let Some(name) = module_name.strip_prefix('%') {
            Module::flipflop(name, destinations)
        } else if let Some(name) = module_name.strip_prefix('&') {
            Module::conjunction(name, destinations)
        } else {
            Err(DayError::ParseError(value.to_owned()))
        }
    }

    fn new(modules: Vec<Module<'a>>) -> Result<Self, DayError> {
        let config = Self { modules };

        for source in config.modules.iter() {
            for dest_name in source.get_destinations() {
                if let Some(dest) = config.find(dest_name) {
                    dest.announce_source(source.name());
                }
            }
        }

        Ok(config)
    }

    pub fn find(&self, name: &str) -> Option<&Module<'a>> {
        self.modules.iter().find(|m| m.name() == name)
    }

    pub fn calc_pulses(&mut self, rounds: usize) -> (usize, usize) {
        let (real, low, high) = self.press_repeat(rounds);
        (low * rounds / real, high * rounds / real)
    }

    pub fn press_repeat(&mut self, max_round: usize) -> (usize, usize, usize) {
        let mut high = 0;
        let mut low = 0;
        for round in 1..=max_round {
            let (next_low, next_high) = self.press_button(|_| {});
            high += next_high;
            low += next_low;
            if self.is_at_start() {
                return (round, low, high);
            }
        }
        (max_round, low, high)
    }

    pub fn press_button<F>(&mut self, inform_receiver: F) -> (usize, usize)
    where
        F: Fn(Pulse),
    {
        let mut low = 1;
        let mut high = 0;
        let mut queue = VecDeque::new();
        queue.push_back((BROADCASTER, BUTTON, Pulse::Low));
        while let Some((module_name, source_name, pulse)) = queue.pop_front() {
            let Some(module) = self.find(module_name) else {
                continue;
            };

            let pulse = module.handle_pulse(source_name, pulse);
            for dest_name in module.get_destinations() {
                match pulse {
                    Pulse::High => high += 1,
                    Pulse::Low => low += 1,
                }
                if let Some(dest_module) = self.find(dest_name) {
                    if dest_module.will_work_on_pulse(pulse) {
                        queue.push_back((dest_name, module_name, pulse));
                    }
                } else {
                    inform_receiver(pulse)
                }
            }
        }
        (low, high)
    }

    fn is_at_start(&self) -> bool {
        self.modules.iter().all(|m| m.is_at_start_state())
    }

    fn count_pushes(&mut self) -> usize {
        for p in 1.. {
            let do_continue = Cell::new(true);
            self.press_button(|pulse| {
                if matches!(pulse, Pulse::Low) {
                    do_continue.set(false);
                }
            });
            if !do_continue.get() {
                return p;
            }
        }
        unreachable!()
    }
}

impl<'a> TryFrom<&'a str> for Configuration<'a> {
    type Error = DayError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let modules = value
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(Configuration::create_module)
            .chain(std::iter::once(Configuration::create_button()))
            .try_collect()?;
        Self::new(modules)
    }
}

struct ComplexSolver<'a> {
    configuration: Configuration<'a>,
}

impl<'a> ComplexSolver<'a> {
    pub fn solve(configuration: Configuration<'a>) -> Result<usize, DayError> {
        let solver = ComplexSolver { configuration };
        let bc = solver
            .configuration
            .find(BROADCASTER)
            .ok_or(DayError::NoBroadcaster)?;

        let mut rounds = 1;
        for split in bc.get_destinations() {
            let sub_modules = solver.collect(split);
            let mut sub_config = Configuration::new(sub_modules)?;
            let pushes = sub_config.count_pushes();
            rounds = lcm(rounds, pushes);
        }
        Ok(rounds)
    }

    fn collect(&'a self, start: &'a str) -> Vec<Module<'a>> {
        let mut queue = vec![start];
        let mut names = vec![];
        while let Some(name) = queue.pop() {
            if names.contains(&name) {
                continue;
            }
            names.push(name);
            let Some(module) = self.configuration.find(name) else {
                continue;
            };
            if !module.is_flipflop() {
                continue;
            }
            queue.extend(module.get_destinations())
        }

        names
            .into_iter()
            .map(|name| self.configuration.find(name).cloned())
            .chain(std::iter::once(
                Module::relay(BROADCASTER, vec![start]).ok(),
            ))
            .flatten()
            .collect_vec()
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
        let expected = ResultType::Integer(32000000);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part1a() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;
        let expected = ResultType::Integer(11687500);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn press_once() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;

        let mut config: Configuration = input.as_str().try_into()?;
        assert_eq!(config.press_button(|_| {}), (8, 4));

        Ok(())
    }

    #[test]
    fn press_repeat() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;

        let mut config: Configuration = input.as_str().try_into()?;
        assert_eq!(config.press_repeat(2), (1, 8, 4));

        Ok(())
    }

    #[test]
    fn press_repeat2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;

        let mut config: Configuration = input.as_str().try_into()?;
        assert_eq!(config.press_repeat(1_000), (4, 17, 11));

        Ok(())
    }

    #[test]
    fn calc_pulses() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;

        let mut config: Configuration = input.as_str().try_into()?;
        assert_eq!(config.calc_pulses(3), (13, 9));

        Ok(())
    }
}
