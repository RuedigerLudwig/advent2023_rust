#![allow(dead_code)]
use super::pos2::Pos2;
use num_traits::{Num, NumAssignOps};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Area<T>
where
    T: Num,
{
    lower_right: Pos2<T>,
    upper_left: Pos2<T>,
}

impl<T> Area<T>
where
    T: Num + Ord + Copy,
{
    pub fn new(p1: Pos2<T>, p2: Pos2<T>) -> Area<T> {
        Area {
            lower_right: p1.max_components(p2),
            upper_left: p1.min_components(p2),
        }
    }

    pub fn from_points(x1: T, y1: T, x2: T, y2: T) -> Area<T> {
        Area {
            lower_right: Pos2::new(x1.max(x2), y1.max(y2)),
            upper_left: Pos2::new(x1.min(x2), y1.min(y2)),
        }
    }
}

impl<T> Area<T>
where
    T: Num + Ord + Copy,
{
    pub fn extend(&self, pos: Pos2<T>) -> Area<T> {
        if self.contains(pos) {
            return *self;
        }

        Area {
            lower_right: self.lower_right.max_components(pos),
            upper_left: self.upper_left.min_components(pos),
        }
    }
    pub fn lower_right(&self) -> Pos2<T> {
        self.lower_right
    }

    pub fn upper_left(&self) -> Pos2<T> {
        self.upper_left
    }

    pub fn upper_right(&self) -> Pos2<T> {
        Pos2::new(self.lower_right.x(), self.upper_left.y())
    }

    pub fn lower_left(&self) -> Pos2<T> {
        Pos2::new(self.upper_left.x(), self.lower_right.y())
    }

    pub fn contains(&self, pos: Pos2<T>) -> bool {
        (self.left()..=self.right()).contains(&pos.x())
            && (self.top()..=self.bottom()).contains(&pos.y())
    }

    pub fn widen(self, inc: T) -> Self {
        Self::new(
            self.lower_right - Pos2::splat(inc),
            self.upper_left + Pos2::splat(inc),
        )
    }
}

impl<'a, T> Area<T>
where
    T: Num + Ord + 'a + Copy,
{
    pub fn from_iterator<I>(mut iter: I) -> Option<Self>
    where
        I: Iterator<Item = &'a Pos2<T>>,
    {
        let first = *iter.next()?;
        let (upper, lower) = iter.fold((first, first), |(mx, mn), p| {
            (mx.max_components(*p), mn.min_components(*p))
        });

        Some(Area::new(lower, upper))
    }
}

impl<T> Area<T>
where
    T: Num + Copy,
{
    #[inline]
    pub fn top(&self) -> T {
        self.upper_left.y()
    }

    #[inline]
    pub fn right(&self) -> T {
        self.lower_right.x()
    }

    #[inline]
    pub fn bottom(&self) -> T {
        self.lower_right.y()
    }

    #[inline]
    pub fn left(&self) -> T {
        self.upper_left.x()
    }

    pub fn width(&self) -> T {
        self.right() - self.left() + T::one()
    }

    pub fn height(&self) -> T {
        self.bottom() - self.top() + T::one()
    }
}

impl<T> Area<T>
where
    T: Num + Copy,
{
    #[allow(dead_code)]
    pub fn area(&self) -> T {
        self.width() * self.height()
    }
}

impl<T> Display for Area<T>
where
    T: Num + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}-{}]", self.lower_right, self.upper_left)
    }
}

impl<T> Area<T>
where
    T: Num + Copy,
{
    pub fn cells(&self, ascending: bool) -> CellIterator<'_, T> {
        CellIterator::new(self, ascending)
    }

    pub fn rows(&self, ascending: bool) -> RowIterator<'_, T> {
        RowIterator::new(self, ascending)
    }
}

#[derive(Debug)]
pub struct RowIterator<'a, T>
where
    T: Num + Copy,
{
    area: &'a Area<T>,
    row: T,
    ascending: bool,
}

impl<'a, T> RowIterator<'a, T>
where
    T: Num + Copy,
{
    fn new(area: &'a Area<T>, ascending: bool) -> RowIterator<'a, T> {
        RowIterator {
            area,
            row: if ascending { area.bottom() } else { area.top() },
            ascending,
        }
    }
}

impl<'a, T> Iterator for RowIterator<'a, T>
where
    T: Num + Ord + NumAssignOps + Copy,
{
    type Item = Row<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.ascending && self.row >= self.area.top())
            || (!self.ascending && self.row <= self.area.bottom())
        {
            let row = Row {
                area: self.area,
                row: self.row,
            };
            if self.ascending {
                self.row += T::one();
            } else {
                self.row -= T::one();
            }
            Some(row)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Row<'a, T>
where
    T: Num,
{
    area: &'a Area<T>,
    row: T,
}

impl<'a, T> Row<'a, T>
where
    T: Num + Copy,
{
    pub fn cols(&self, ascending: bool) -> ColIterator<'_, T> {
        ColIterator {
            area: self.area,
            row: self.row,
            col: if ascending {
                self.area.left()
            } else {
                self.area.right()
            },
            ascending,
        }
    }
}

#[derive(Debug)]
pub struct ColIterator<'a, T>
where
    T: Num,
{
    area: &'a Area<T>,
    row: T,
    col: T,
    ascending: bool,
}

impl<'a, T> Iterator for ColIterator<'a, T>
where
    T: Num + Ord + NumAssignOps + Copy,
{
    type Item = Pos2<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if (self.ascending && self.col <= self.area.right())
            || (!self.ascending && self.col >= self.area.left())
        {
            let pos = Pos2::new(self.col, self.row);
            if self.ascending {
                self.col += T::one();
            } else {
                self.col -= T::one();
            }
            Some(pos)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct CellIterator<'a, T>
where
    T: Num,
{
    area: &'a Area<T>,
    row: T,
    col: T,
    ascending: bool,
}

impl<'a, T> CellIterator<'a, T>
where
    T: Num + Copy,
{
    pub fn new(area: &'a Area<T>, ascending: bool) -> CellIterator<'a, T> {
        let (col, row) = if ascending {
            (area.left(), area.bottom())
        } else {
            (area.right(), area.top())
        };
        CellIterator {
            area,
            row,
            col,
            ascending,
        }
    }
}

impl<'a, T> Iterator for CellIterator<'a, T>
where
    T: Num + Ord + NumAssignOps + Copy,
{
    type Item = Pos2<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.ascending && self.row >= self.area.top())
            || (!self.ascending && self.row <= self.area.bottom())
        {
            let pos = Pos2::new(self.col, self.row);
            if self.ascending {
                self.col += T::one();
                if self.col > self.area.right() {
                    self.row -= T::one();
                    self.col = self.area.left();
                }
            } else {
                self.col -= T::one();
                if self.col < self.area.left() {
                    self.row += T::one();
                    self.col = self.area.right();
                }
            }

            Some(pos)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cell_iterator() {
        let area = Area::new(Pos2::new(-1, -1), Pos2::new(1, 1));
        let result = area.cells(true).collect::<Vec<_>>();
        let expected = vec![
            Pos2::new(-1, -1),
            Pos2::new(0, -1),
            Pos2::new(1, -1),
            Pos2::new(-1, 0),
            Pos2::new(0, 0),
            Pos2::new(1, 0),
            Pos2::new(-1, 1),
            Pos2::new(0, 1),
            Pos2::new(1, 1),
        ];
        assert_eq!(result, expected);
    }
}
