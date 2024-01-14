use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    fmt::Display,
    num,
    str::FromStr,
};

const DAY_NUMBER: DayType = 20;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let config: Configuration = input.parse()?;
        let (low, high) = config.calc_pulses(1_000);
        Ok((low * high).into())
    }

    fn part2(&self, input: &str) -> RResult {
        Ok(().into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
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

trait Module {
    fn name(&self) -> &str;
    fn send_pulse(&self, source: &str, pulse: Pulse) -> bool;
    fn generate(&self) -> Pulse;
    fn announce(&self, source: &str);
    fn get_connections(&self) -> &[String];
    fn at_start(&self) -> bool;
}
const BUTTON: &str = "button";
const BROARDCASTER: &str = "broadcaster";

struct Button {
    connections: Vec<String>,
}

impl Button {
    pub fn new() -> Self {
        Self {
            connections: vec![String::from(BROARDCASTER)],
        }
    }
}

impl Module for Button {
    fn name(&self) -> &str {
        BUTTON
    }

    fn send_pulse(&self, _source: &str, _pulse: Pulse) -> bool {
        false
    }

    fn generate(&self) -> Pulse {
        Pulse::Low
    }

    fn announce(&self, _source: &str) {}

    fn get_connections(&self) -> &[String] {
        &self.connections
    }

    fn at_start(&self) -> bool {
        true
    }
}

struct Broadcaster {
    pulse: Cell<Pulse>,
    connections: Vec<String>,
}

impl Broadcaster {
    pub fn new(connections: Vec<String>) -> Self {
        Self {
            pulse: Cell::new(Pulse::Low),
            connections,
        }
    }
}

impl Module for Broadcaster {
    fn name(&self) -> &str {
        BROARDCASTER
    }

    fn send_pulse(&self, _source: &str, pulse: Pulse) -> bool {
        self.pulse.set(pulse);
        true
    }

    fn generate(&self) -> Pulse {
        self.pulse.get()
    }

    fn announce(&self, _source: &str) {}

    fn get_connections(&self) -> &[String] {
        &self.connections
    }

    fn at_start(&self) -> bool {
        true
    }
}

struct FlipFlop {
    name: String,
    is_on: Cell<bool>,
    connections: Vec<String>,
}

impl FlipFlop {
    pub fn new(name: &str, connections: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            is_on: Cell::new(false),
            connections,
        }
    }
}

impl Module for FlipFlop {
    fn name(&self) -> &str {
        &self.name
    }

    fn send_pulse(&self, _source: &str, pulse: Pulse) -> bool {
        match pulse {
            Pulse::High => false,
            Pulse::Low => {
                self.is_on.set(!self.is_on.get());
                true
            }
        }
    }

    fn generate(&self) -> Pulse {
        if self.is_on.get() {
            Pulse::High
        } else {
            Pulse::Low
        }
    }

    fn announce(&self, _source: &str) {}

    fn get_connections(&self) -> &[String] {
        &self.connections
    }

    fn at_start(&self) -> bool {
        !self.is_on.get()
    }
}

struct Conjunction {
    name: String,
    prev: RefCell<HashMap<String, Pulse>>,
    connections: Vec<String>,
}

impl Conjunction {
    pub fn new(name: &str, connections: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            prev: RefCell::new(HashMap::new()),
            connections,
        }
    }
}

impl Module for Conjunction {
    fn name(&self) -> &str {
        &self.name
    }

    fn send_pulse(&self, source: &str, pulse: Pulse) -> bool {
        let mut prev = self.prev.borrow_mut();
        prev.insert(source.to_owned(), pulse);
        true
    }

    fn generate(&self) -> Pulse {
        if self
            .prev
            .borrow()
            .values()
            .all(|p| matches!(p, Pulse::High))
        {
            Pulse::Low
        } else {
            Pulse::High
        }
    }
    fn announce(&self, source: &str) {
        self.prev
            .borrow_mut()
            .insert(source.to_string(), Pulse::Low);
    }

    fn get_connections(&self) -> &[String] {
        &self.connections
    }

    fn at_start(&self) -> bool {
        self.prev.borrow().values().all(|p| matches!(p, Pulse::Low))
    }
}

struct Configuration {
    modules: Vec<Box<dyn Module>>,
}

impl Configuration {
    fn create_button() -> Result<Box<dyn Module>, DayError> {
        Ok(Box::new(Button::new()))
    }

    fn create_module(value: &str) -> Result<Box<dyn Module>, DayError> {
        let Some((module_name, connections)) = value.split_once("->") else {
            return Err(DayError::ParseError(value.to_owned()));
        };
        let connections = connections
            .split(',')
            .map(|c| c.trim().to_string())
            .collect_vec();

        let module_name = module_name.trim();
        let module: Box<dyn Module> = if module_name == BROARDCASTER {
            Box::new(Broadcaster::new(connections))
        } else if let Some(name) = module_name.strip_prefix('%') {
            Box::new(FlipFlop::new(name, connections))
        } else if let Some(name) = module_name.strip_prefix('&') {
            Box::new(Conjunction::new(name, connections))
        } else {
            return Err(DayError::ParseError(value.to_owned()));
        };
        Ok(module)
    }

    fn new(modules: Vec<Box<dyn Module>>) -> Result<Configuration, DayError> {
        let config = Self { modules };
        for source in config.modules.iter() {
            for dest_name in source.get_connections() {
                if let Some(dest) = config.find(dest_name) {
                    dest.announce(source.name());
                }
            }
        }
        Ok(config)
    }

    pub fn find(&self, name: &str) -> Option<&Box<dyn Module>> {
        self.modules.iter().find(|m| m.name() == name)
    }

    pub fn calc_pulses(&self, rounds: usize) -> (usize, usize) {
        let (real, low, high) = self.press_repeat(rounds);
        (low * rounds / real, high * rounds / real)
    }

    pub fn press_repeat(&self, max_round: usize) -> (usize, usize, usize) {
        let mut high = 0;
        let mut low = 0;
        for round in 1..=max_round {
            let (next_low, next_high) = self.press_button();
            high += next_high;
            low += next_low;
            if self.is_at_start() {
                return (round, low, high);
            }
        }
        (max_round, low, high)
    }

    pub fn press_button(&self) -> (usize, usize) {
        let mut low = 0;
        let mut high = 0;
        let mut queue = VecDeque::new();
        queue.push_back(String::from(BUTTON));
        while let Some(source_name) = queue.pop_front() {
            let Some(source) = self.find(&source_name) else {
                continue;
            };
            let pulse = source.generate();
            for dest_name in source.get_connections() {
                //println!("{} -{}-> {}", source_name, pulse, dest_name);
                match pulse {
                    Pulse::High => high += 1,
                    Pulse::Low => low += 1,
                }
                if let Some(dest_module) = self.find(dest_name) {
                    if dest_module.send_pulse(&source_name, pulse) && !queue.contains(dest_name) {
                        queue.push_back(dest_name.to_string());
                    }
                } else {
                    //println!("Not found: {}", dest_name);
                }
            }
        }
        (low, high)
    }

    fn is_at_start(&self) -> bool {
        self.modules.iter().all(|m| m.at_start())
    }
}

impl FromStr for Configuration {
    type Err = DayError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let modules = value
            .lines()
            .map(Configuration::create_module)
            .chain(std::iter::once(Configuration::create_button()))
            .try_collect()?;
        Self::new(modules)
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
        // 797028600 too low
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;
        let expected = ResultType::Integer(11687500);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Nothing;
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn press_once() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;

        let config: Configuration = input.parse()?;
        assert_eq!(config.press_button(), (8, 4));

        Ok(())
    }

    #[test]
    fn press_repeat() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;

        let config: Configuration = input.parse()?;
        assert_eq!(config.press_repeat(2), (1, 8, 4));

        Ok(())
    }

    #[test]
    fn press_repeat2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;

        let config: Configuration = input.parse()?;
        assert_eq!(config.press_repeat(1_000), (4, 17, 11));

        Ok(())
    }

    #[test]
    fn calc_pulses() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;

        let config: Configuration = input.parse()?;
        assert_eq!(config.calc_pulses(3), (13, 9));

        Ok(())
    }
}
