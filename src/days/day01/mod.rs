use super::{DayTrait, DayType, RResult};

const DAY_NUMBER: DayType = 1;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let result: u32 = day_impl::get_digits(input).sum();
        Ok(result.into())
    }

    fn part2(&self, input: &str) -> RResult {
        let result: u32 = day_impl::get_worded_digits(input).sum();
        Ok(result.into())
    }
}

mod day_impl {
    pub fn get_digits(input: &str) -> impl Iterator<Item = u32> + '_ {
        input.lines().filter_map(convert)
    }

    pub fn get_worded_digits(input: &str) -> impl Iterator<Item = u32> + '_ {
        input
            .lines()
            .map(replace_number_words)
            .filter_map(|line| convert(&line))
    }

    /**
     * No need to be fance her, just walk throug the string and remember
     * the first and (so far) last seen digits
     */
    fn convert(line: &str) -> Option<u32> {
        line.chars()
            .fold(None, |prev, c| {
                let Some(digit) = c.to_digit(10) else {
                    return prev;
                };

                match prev {
                    None => Some((digit, digit)),
                    Some((first, _)) => Some((first, digit)),
                }
            })
            .map(|(first, last)| first * 10 + last)
    }

    /**
     * Replace the word with their digits. Also add letters that could potentially
     * be part of following numbers
     */
    fn replace_number_words(line: &str) -> String {
        line.replace("one", "o1e")
            .replace("two", "t2")
            .replace("three", "t3e")
            .replace("four", "4")
            .replace("five", "5e")
            .replace("six", "6")
            .replace("seven", "7n")
            .replace("eight", "8")
            .replace("nine", "9")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::days::{read_string, ResultType, UnitResult};
    use itertools::Itertools;

    #[test]
    fn test_part1() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(142);
        let result = day.part1(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_part2() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;
        let expected = ResultType::Integer(281);
        let result = day.part2(&input)?;
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn get_digits() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = [12, 38, 15, 77];
        assert_eq!(day_impl::get_digits(&input).collect_vec(), expected);

        Ok(())
    }

    #[test]
    fn get_worded_digits() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example02.txt")?;
        let expected = [29, 83, 13, 24, 42, 14, 76];
        assert_eq!(day_impl::get_worded_digits(&input).collect_vec(), expected);

        Ok(())
    }
}
