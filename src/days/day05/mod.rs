use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use std::{num, ops::Range, str::FromStr};

const DAY_NUMBER: DayType = 5;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let almanach: Almanach = input.parse()?;
        let locations = almanach.all_locations().into_iter().min().unwrap();
        Ok(locations.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let almanach: Almanach = input.parse()?;
        let locations = almanach.range_location();
        Ok(locations.into())
    }
}

#[derive(Debug, thiserror::Error)]
enum DayError {
    #[error("Not a valid description: {0}")]
    ParseError(String),
    #[error("Not an Int")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("No Mapping was given to us")]
    NoMappingGiven,
}

fn range_overlaps(first: &Range<u64>, second: &Range<u64>) -> bool {
    first.start < second.end && second.start < first.end
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct RangeMappings {
    source: Range<u64>,
    dest: Range<u64>,
}

impl RangeMappings {
    pub fn new_equal(start: u64, end: u64) -> Self {
        Self {
            source: start..end,
            dest: start..end,
        }
    }

    pub fn new(dest: u64, source: u64, len: u64) -> Self {
        Self {
            source: source..source + len,
            dest: dest..dest + len,
        }
    }
}

impl PartialOrd for RangeMappings {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RangeMappings {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.dest.start.cmp(&other.dest.start)
    }
}

impl FromStr for RangeMappings {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nums: Vec<u64> = s
            .split_ascii_whitespace()
            .map(|num| num.parse())
            .try_collect()?;

        if nums.len() != 3 {
            return Err(DayError::ParseError(s.to_owned()));
        }

        Ok(Self::new(nums[0], nums[1], nums[2]))
    }
}

impl RangeMappings {
    pub fn convert_source_dest(&self, source: u64) -> Option<u64> {
        if self.source.contains(&source) {
            Some(source - self.source.start + self.dest.start)
        } else {
            None
        }
    }

    pub fn convert_dest_source(&self, dest: u64) -> Option<u64> {
        if self.dest.contains(&dest) {
            Some(dest - self.dest.start + self.source.start)
        } else {
            None
        }
    }

    pub fn possible_dest_split(&self, dest: &Range<u64>) -> Option<Range<u64>> {
        if range_overlaps(&self.dest, dest) {
            let start = self
                .convert_dest_source(dest.start)
                .unwrap_or(self.source.start);
            let end = self
                .convert_dest_source(dest.end)
                .unwrap_or(self.source.end);
            Some(start..end)
        } else {
            None
        }
    }
}

struct Mapping {
    ranges: Vec<RangeMappings>,
}

impl Mapping {
    pub fn new(ranges: Vec<RangeMappings>) -> Self {
        let (last, mut ranges) =
            ranges
                .into_iter()
                .sorted()
                .fold((0, vec![]), |(last, mut ranges), range| {
                    let end = range.dest.end;
                    if range.dest.start > last {
                        ranges.push(RangeMappings::new_equal(last, range.dest.start));
                    }
                    ranges.push(range);
                    (end, ranges)
                });
        ranges.push(RangeMappings::new_equal(last, u64::MAX));
        Self { ranges }
    }

    pub fn gather<'a>(iter: &mut impl Iterator<Item = &'a str>) -> Result<Option<Self>, DayError> {
        let Some(description) = iter.next() else {
            return Ok(None);
        };
        let Some(names) = description.strip_suffix(" map:") else {
            return Err(DayError::ParseError(description.to_owned()));
        };
        let names: Vec<_> = names.split('-').collect();
        if names.len() != 3 {
            return Err(DayError::ParseError(description.to_owned()));
        }

        let mappings = iter
            .take_while(|line| !line.is_empty())
            .map(|line| line.parse())
            .try_collect()?;

        Ok(Some(Self::new(mappings)))
    }

    pub fn convert(&self, source: u64) -> u64 {
        self.ranges
            .iter()
            .filter_map(|range| range.convert_source_dest(source))
            .next()
            .unwrap_or(source)
    }

    pub fn possible_dest_split(&self, dest: &Range<u64>) -> Vec<Range<u64>> {
        self.ranges
            .iter()
            .filter_map(|range| range.possible_dest_split(dest))
            .collect_vec()
    }
}

struct Almanach {
    seeds: Vec<u64>,
    mappings: Vec<Mapping>,
}

impl Almanach {
    pub fn one_location(&self, seed: u64) -> u64 {
        self.mappings
            .iter()
            .fold(seed, |item, map| map.convert(item))
    }

    pub fn all_locations(&self) -> Vec<u64> {
        self.seeds
            .iter()
            .map(|seed| self.one_location(*seed))
            .collect_vec()
    }

    pub fn possible_dest_split(&self, dest: Range<u64>) -> Vec<Range<u64>> {
        self.mappings.iter().rev().fold(vec![dest], |ranges, map| {
            ranges
                .iter()
                .flat_map(|dest_range| map.possible_dest_split(dest_range))
                .collect_vec()
        })
    }

    pub fn range_location(&self) -> u64 {
        self.mappings
            .last()
            .expect("This can never happen. We have at least one mapping")
            .ranges
            .iter()
            .find_map(|range| {
                let seeds = self.possible_dest_split(range.dest.clone());
                let result = seeds
                    .into_iter()
                    .flat_map(|ps| {
                        self.seeds
                            .iter()
                            .tuples()
                            .filter_map(move |(&start, &len)| {
                                let seed = start..start + len;
                                if seed.contains(&ps.start) {
                                    Some(ps.start)
                                } else if ps.contains(&seed.start) {
                                    Some(seed.start)
                                } else {
                                    None
                                }
                            })
                    })
                    .collect_vec();
                if result.is_empty() {
                    None
                } else {
                    Some(result)
                }
            })
            .expect("This can never happen we will always have a at least one item")
            .into_iter()
            .map(|seed| self.one_location(seed))
            .min()
            .expect("This can never happend - we amde sure we have at least one item")
    }
}

impl FromStr for Almanach {
    type Err = DayError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut lines = value.lines();
        let Some(seeds) = lines.next() else {
            return Err(DayError::ParseError(value.to_owned()));
        };
        let Some(seeds) = seeds.strip_prefix("seeds: ") else {
            return Err(DayError::ParseError(value.to_owned()));
        };
        let seeds = seeds
            .split_ascii_whitespace()
            .map(|seed| seed.parse())
            .try_collect()?;

        let _ = lines.next();

        let mut mappings = vec![];
        while let Some(mapping) = Mapping::gather(&mut lines)? {
            mappings.push(mapping);
        }

        if mappings.is_empty() {
            Err(DayError::NoMappingGiven)
        } else {
            Ok(Self { seeds, mappings })
        }
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
        let expected = ResultType::Integer(35);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(46);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn parse() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let almanach: Almanach = input.parse()?;

        assert_eq!(almanach.seeds, [79, 14, 55, 13]);
        assert_eq!(almanach.mappings.len(), 7);
        assert_eq!(almanach.mappings[6].ranges.len(), 4);
        assert_eq!(
            almanach.mappings[6].ranges[0],
            RangeMappings {
                source: 0..56,
                dest: 0..56,
            }
        );
        assert_eq!(
            almanach.mappings[6].ranges[1],
            RangeMappings {
                source: 93..97,
                dest: 56..60,
            }
        );
        assert_eq!(almanach.mappings[0].convert(79), 81);
        assert_eq!(almanach.mappings[0].convert(14), 14);
        assert_eq!(almanach.all_locations(), [82, 43, 86, 35]);

        Ok(())
    }

    #[test]
    fn split() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let almanach: Almanach = input.parse()?;

        assert_eq!(
            almanach.mappings[0].ranges[2].possible_dest_split(&(42..62)),
            Some(50..60)
        );

        assert_eq!(
            almanach.mappings[1].possible_dest_split(&(37..42)),
            vec![52..54, 0..3]
        );

        Ok(())
    }

    #[test]
    fn possible() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let almanach: Almanach = input.parse()?;

        let ranges = almanach.possible_dest_split(46..47);
        assert!(ranges.iter().any(|range| range.contains(&82)));

        assert_eq!(almanach.range_location(), 46);

        Ok(())
    }
}
