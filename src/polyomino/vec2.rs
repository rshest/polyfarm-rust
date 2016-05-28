use std::ops::{Add, Sub};
use num::Num;

pub type Vec2i = Vec2<i32>;
pub type Vec2f = Vec2<f64>;

#[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

// implementation
impl<T: Num> Vec2<T> {
    pub fn new(x: T, y: T) -> Vec2<T> {
        Vec2 { x: x, y: y }
    } 
}

impl Vec2<i32> {
    pub fn len(&self) -> f64 {
        ((self.x*self.x + self.y*self.y) as f64).sqrt()
    }
}

// compare with [x, y]
impl<T: Num> PartialEq<[T; 2]> for Vec2<T> {
    #[inline]
    fn eq(&self, rhs: &[T; 2]) -> bool {
        self.x == rhs[0] && self.y == rhs[1]
    }
}

// operator +
impl<T: Num> Add for Vec2<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Vec2<T>) -> Self {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

// operator -
impl<T: Num> Sub for Vec2<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Vec2<T>) -> Self {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetics() {
        let a = Vec2i { x: -1, y: 5 };
        let b = Vec2i { x: 3, y: -2 };

        assert_eq!(Vec2i { x: 2, y: 3 }, a + b);
        assert_eq!(Vec2i { x: 4, y: -7 }, b - a);
    }
}
