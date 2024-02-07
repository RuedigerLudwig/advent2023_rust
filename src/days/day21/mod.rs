use super::{DayTrait, DayType, RResult};
use crate::common::{direction::Direction, pos2::Pos2};
use itertools::Itertools;
use std::{collections::HashSet, fmt::Display, num, str::FromStr};

const DAY_NUMBER: DayType = 21;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let map: GardenMap = input.parse()?;
        let possible = map.do_few_steps();
        Ok(possible.len().into())
    }

    fn part2(&self, input: &str) -> RResult {
        let map: GardenMap = input.parse()?;
        let plots = map.do_many_steps()?;
        Ok(plots.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Unknown PLot: {0}")]
    UnknownPlot(char),
    #[error("There must be exactly one start")]
    ExactlyOneStart,
    #[error("Plots must never be empty")]
    PlotsMustNotBeEmpty,
    #[error("Plots must be a rectangle")]
    PlotsMustBeARectangle,
    #[error("The algorithm does not work on this input")]
    AlgorithmDoesNotWork,
}

#[derive(Debug, Clone, Copy)]
enum Plot {
    Garden,
    Rock,
    Start,
}

impl Display for Plot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Plot::Garden => ' ',
                Plot::Rock => '#',
                Plot::Start => 'S',
            }
        )
    }
}

impl Plot {
    pub fn is_garden(&self) -> bool {
        !matches!(self, Plot::Rock)
    }

    pub fn is_start(&self) -> bool {
        matches!(self, Plot::Start)
    }
}

impl TryFrom<char> for Plot {
    type Error = DayError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Plot::Garden),
            '#' => Ok(Plot::Rock),
            'S' => Ok(Plot::Start),
            _ => Err(DayError::UnknownPlot(value)),
        }
    }
}

struct GardenMap {
    few_steps: usize,
    many_steps: usize,
    start: Pos2<usize>,
    plots: Vec<Vec<Plot>>,
}

impl FromStr for GardenMap {
    type Err = DayError;

    fn from_str(lines: &str) -> Result<Self, Self::Err> {
        let Some((steps, map)) = lines.split_once('\n') else {
            return Err(DayError::ParseError(lines.to_owned()));
        };
        let Some((steps1, steps2)) = steps.split_once('/') else {
            return Err(DayError::ParseError(steps.to_owned()));
        };
        let few_steps = steps1.trim().parse()?;
        let many_steps = steps2.trim().parse()?;

        let plots: Vec<Vec<Plot>> = map
            .lines()
            .map(|row| row.chars().map(|plot| plot.try_into()).try_collect())
            .try_collect()?;
        let start = plots
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, plot)| {
                    if plot.is_start() {
                        Some(Pos2::new(x, y))
                    } else {
                        None
                    }
                })
            })
            .exactly_one()
            .map_err(|_| DayError::ExactlyOneStart)?;
        Self::new(plots, start, few_steps, many_steps)
    }
}

impl GardenMap {
    pub fn do_few_steps(&self) -> HashSet<Pos2<usize>> {
        self.calc_steps_from_single(self.few_steps, self.start)
    }

    #[inline]
    fn calc_steps_from_single(&self, steps: usize, start: Pos2<usize>) -> HashSet<Pos2<usize>> {
        let mut positions = HashSet::new();
        positions.insert(start);
        self.calc_steps_from_multi(steps, positions)
    }

    fn calc_steps_from_multi(
        &self,
        steps: usize,
        mut positions: HashSet<Pos2<usize>>,
    ) -> HashSet<Pos2<usize>> {
        for _ in 0..steps {
            positions = self.calc_one_step(positions);
        }

        positions
    }

    fn calc_one_step(&self, positions: HashSet<Pos2<usize>>) -> HashSet<Pos2<usize>> {
        let mut next_positions = HashSet::new();
        for pos in positions {
            for dir in Direction::iter() {
                if let Some((p, pl)) = pos.safe_matrix_add_and_get(&self.plots, dir) {
                    if pl.is_garden() {
                        next_positions.insert(p);
                    }
                }
            }
        }
        next_positions
    }

    fn do_many_steps(&self) -> Result<usize, DayError> {
        self.calc_many_steps(self.many_steps)
    }

    fn get_small_big(&self, start: Pos2<usize>, half: usize, full: usize) -> (usize, usize) {
        let positions = self.calc_steps_from_single(half, start);
        let small = positions.len();
        let positions = self.calc_steps_from_multi(full, positions);
        let big = positions.len();
        (small, big)
    }

    fn calc_many_steps(&self, steps: usize) -> Result<usize, DayError> {
        let full = self.plots.len();
        let half = self.start.x();
        if self.start.y() != half || half * 2 + 1 != full {
            return Err(DayError::AlgorithmDoesNotWork);
        }
        if full != self.plots[0].len() {
            return Err(DayError::AlgorithmDoesNotWork);
        }
        if self.plots.iter().any(|row| !row[half].is_garden())
            || self.plots[half].iter().any(|plot| !plot.is_garden())
        {
            return Err(DayError::AlgorithmDoesNotWork);
        }
        if steps % full != half {
            return Err(DayError::AlgorithmDoesNotWork);
        }

        let reached = self.calc_steps_from_single(full, self.start);
        let one = reached.len();
        let reached = self.calc_one_step(reached);
        let two = reached.len();

        let full_squares = steps / full - 1;
        let last_squares = full_squares * 2 + 1;
        let full_reached = full_squares.pow(2) * (one + two) + last_squares * two;

        let le = self
            .calc_steps_from_single(full - 1, Pos2::new(0, half))
            .len();
        let lw = self
            .calc_steps_from_single(full - 1, Pos2::new(full - 1, half))
            .len();
        let ln = self
            .calc_steps_from_single(full - 1, Pos2::new(half, full - 1))
            .len();
        let ls = self
            .calc_steps_from_single(full - 1, Pos2::new(half, 0))
            .len();

        let (sse, bse) = self.get_small_big(Pos2::new(0, 0), half - 1, full);
        let (sne, bne) = self.get_small_big(Pos2::new(0, full - 1), half - 1, full);
        let (ssw, bsw) = self.get_small_big(Pos2::new(full - 1, 0), half - 1, full);
        let (snw, bnw) = self.get_small_big(Pos2::new(full - 1, full - 1), half - 1, full);

        let points = le + ln + lw + ls;
        let border =
            (full_squares + 1) * (sse + sne + ssw + snw) + full_squares * (bse + bne + bsw + bnw);

        Ok(full_reached + points + border)
    }

    fn new(
        plots: Vec<Vec<Plot>>,
        start: Pos2<usize>,
        few_steps: usize,
        many_steps: usize,
    ) -> Result<GardenMap, DayError> {
        if plots.is_empty() || plots[0].is_empty() {
            return Err(DayError::PlotsMustNotBeEmpty);
        }
        if !plots.iter().map(|row| row.len()).all_equal() {
            return Err(DayError::PlotsMustBeARectangle);
        }
        Ok(Self {
            plots,
            start,
            few_steps,
            many_steps,
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
        let expected = ResultType::Integer(16);
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
    fn parse() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let map: GardenMap = input.parse()?;
        assert_eq!(map.few_steps, 6);
        assert_eq!(map.start, Pos2::new(5, 5));

        Ok(())
    }

    #[test]
    fn steps() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let map: GardenMap = input.parse()?;
        let possible = map.calc_steps_from_single(1, map.start);
        assert_eq!(possible.len(), 2);
        let possible = map.calc_steps_from_single(map.few_steps, map.start);
        assert_eq!(possible.len(), 16);

        Ok(())
    }

    fn explode(gm: &GardenMap, factor: usize) -> GardenMap {
        let plots = gm
            .plots
            .iter()
            .map(|row| {
                row.iter()
                    .cycle()
                    .copied()
                    .take(factor * row.len())
                    .collect_vec()
            })
            .cycle()
            .take(gm.plots.len() * factor)
            .collect_vec();
        let start = gm.start + Pos2::new(gm.plots.len(), gm.plots.len()) * (factor / 2);
        GardenMap {
            plots,
            start,
            few_steps: gm.few_steps,
            many_steps: gm.many_steps,
        }
    }

    #[test]
    fn big27() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;
        let map: GardenMap = input.parse()?;

        let steps = 27;
        let factor = steps / (map.plots.len() / 2);
        let map2 = explode(&map, factor);
        let possible = map2.calc_steps_from_single(steps, map2.start);
        let reached = map.calc_many_steps(steps)?;
        assert_eq!(reached, possible.len());

        Ok(())
    }

    #[test]
    fn big38() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;
        let map: GardenMap = input.parse()?;
        let steps = 38;
        let factor = steps / (map.plots.len() / 2);
        let map2 = explode(&map, factor);
        let possible = map2.calc_steps_from_single(steps, map2.start);
        let reached = map.calc_many_steps(steps)?;
        assert_eq!(reached, possible.len());

        Ok(())
    }
}
