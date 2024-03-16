use crate::common::{area::Area, pos2::Pos2, pos3::Pos3};

use super::{DayTrait, DayType, RResult};
use itertools::Itertools;
use num_traits::Zero;
use std::{num, str::FromStr};

const DAY_NUMBER: DayType = 24;

pub struct Day;

impl DayTrait for Day {
    fn get_day_number(&self) -> DayType {
        DAY_NUMBER
    }

    fn part1(&self, input: &str) -> RResult {
        let storm: Hailstorm = input.parse()?;
        Ok(storm.count_collisions().into())
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
    NotAnInt(#[from] num::ParseIntError),
    #[error("Not a Float")]
    NotAtFloat(#[from] num::ParseFloatError),
}

type CoordType = f64;
type PosType = Pos3<CoordType>;

#[derive(Debug, Clone)]
struct Hailstone {
    position: PosType,
    velocity: PosType,
}

impl FromStr for Hailstone {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((p, v)) = s.split_once('@') else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let position: Vec<CoordType> = p.split(',').map(|c| c.trim().parse()).try_collect()?;
        let velocity: Vec<CoordType> = v.split(',').map(|c| c.trim().parse()).try_collect()?;
        Ok(Self {
            position: Pos3::from(position.as_slice()),
            velocity: Pos3::from(velocity.as_slice()),
        })
    }
}

impl Hailstone {
    pub fn intersects_2d<T>(&self, other: &Hailstone, conv: T) -> Option<(Pos2<f64>, f64, f64)>
    where
        T: Fn(Pos3<CoordType>) -> Pos2<f64>,
    {
        let fst_pos = conv(self.position);
        let fst_velocity = conv(self.velocity);
        let snd_pos = conv(other.position);
        let snd_velocity = conv(other.velocity);

        if fst_velocity.is_zero() || snd_velocity.is_zero() {
            if fst_pos == snd_pos {
                return Some((fst_pos, 0.0, 0.0));
            } else {
                return None;
            }
        }

        let px1 = fst_pos.x();
        let py1 = fst_pos.y();
        let vx1 = fst_velocity.x();
        let vy1 = fst_velocity.y();
        let px2 = snd_pos.x();
        let py2 = snd_pos.y();
        let vx2 = snd_velocity.x();
        let vy2 = snd_velocity.y();

        let div = vx2 * vy1 - vy2 * vx1;
        if div == 0.0 || vx1 == 0.0 {
            return None;
        }

        let m = ((py2 - py1) * vx1 - (px2 - px1) * vy1) / (vx2 * vy1 - vy2 * vx1);

        if m == 0.0 {
            return None;
        }
        let n = ((px2 - px1) + m * vx2) / vx1;
        if n == 0.0 {
            return None;
        }

        let pos = Pos2::new(px2 + m * vx2, py2 + m * vy2);
        Some((pos, m, n))
    }
}

struct Hailstorm {
    stones: Vec<Hailstone>,
    test: Area<f64>,
}

impl FromStr for Hailstorm {
    type Err = DayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let Some(first) = lines.next() else {
            return Err(DayError::ParseError(s.to_owned()));
        };
        let Some((x, y)) = first.split_once(',') else {
            return Err(DayError::ParseError(first.to_owned()));
        };
        let Some((x_from, x_to)) = x.split_once('-') else {
            return Err(DayError::ParseError(first.to_owned()));
        };
        let x_from = x_from.parse()?;
        let x_to = x_to.parse()?;
        let Some((y_from, y_to)) = y.split_once('-') else {
            return Err(DayError::ParseError(first.to_owned()));
        };
        let y_from = y_from.parse()?;
        let y_to = y_to.parse()?;

        let stones = lines.map(|line| line.parse()).try_collect()?;

        Ok(Self {
            stones,
            test: Area::create(Pos2::new(x_from, y_from), Pos2::new(x_to, y_to)).unwrap(),
        })
    }
}

impl Hailstorm {
    fn conv_xy(from: Pos3<CoordType>) -> Pos2<f64> {
        let p = from.project_xy();
        Pos2::new(p.x() as f64, p.y() as f64)
    }

    pub fn count_collisions(&self) -> usize {
        self.stones
            .iter()
            .tuple_combinations()
            .filter_map(|(fst, snd)| fst.intersects_2d(snd, Hailstorm::conv_xy))
            .filter(|(point, m, n)| *m >= 0.0 && *n >= 0.0 && self.test.contains(point))
            .count()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        common::matrix3::Matrix3,
        days::{read_string, ResultType, UnitResult},
    };
    use ndarray::prelude::*;
    use ndarray_linalg::Solve;

    #[test]
    fn test_part1() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "example01.txt")?;
        let expected = ResultType::Integer(2);
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
        let storm: Hailstorm = input.parse()?;

        assert_eq!(storm.stones.len(), 5);
        assert_eq!(storm.stones[0].position, Pos3::new(19.0, 13.0, 30.0));
        assert_eq!(storm.stones[0].velocity, Pos3::new(-2.0, 1.0, -2.0));
        if let Some((p, _, _)) = storm.stones[0].intersects_2d(&storm.stones[1], Hailstorm::conv_xy)
        {
            assert_eq!(p, Pos2::new(14.0 + 1.0 / 3.0, 15.0 + 1.0 / 3.0));
        } else {
            panic!("Found None")
        }
        assert_eq!(storm.count_collisions(), 2);

        Ok(())
    }

    #[test]
    fn dummy() -> UnitResult {
        let day = Day {};
        let input = read_string(day.get_day_number(), "input.txt")?;
        let storm: Hailstorm = input.parse()?;

        let s1 = &storm.stones[0];
        let s2 = &storm.stones[1];
        let s3 = &storm.stones[2];

        Ok(())
    }
}
