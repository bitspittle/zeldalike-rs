//! Set of useful, geometry-related utility classes

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// A point in 2D space.
///
/// Note: This struct is so small you should prefer passing it around by copy rather than by
/// reference. See also: https://rust-lang-nursery.github.io/rust-clippy/v0.0.212/index.html#trivially_copy_pass_by_ref
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct P2 {
    pub x: f32,
    pub y: f32,
}

/// A vector in 2D space.
///
/// Points and vectors look identical at first glance, but points represent a location while vectors
/// represent movement. As a result, you can add a vector to a point to get a new point, but you
/// can't add two points together.
///
/// For convenience, you will be able to convert between points and vectors, in case you ever have
/// one and need the other.
///
/// Note: This struct is so small you should prefer passing it around by copy rather than by
/// reference. See also: https://rust-lang-nursery.github.io/rust-clippy/v0.0.212/index.html#trivially_copy_pass_by_ref
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl P2 {
    pub fn new(x: f32, y: f32) -> P2 {
        P2 { x, y }
    }

    pub fn zero() -> P2 {
        P2::new(0., 0.)
    }
}

impl Default for P2 {
    fn default() -> Self {
        P2::zero()
    }
}

impl From<(f32, f32)> for P2 {
    fn from(pair: (f32, f32)) -> Self {
        P2 {
            x: pair.0,
            y: pair.1,
        }
    }
}

impl From<V2> for P2 {
    fn from(vec: V2) -> Self {
        P2::zero() + vec
    }
}

impl Add<V2> for P2 {
    type Output = P2;

    fn add(self, rhs: V2) -> P2 {
        P2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<V2> for P2 {
    type Output = P2;

    fn sub(self, rhs: V2) -> P2 {
        P2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<P2> for P2 {
    type Output = V2;

    fn sub(self, rhs: P2) -> V2 {
        V2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for P2 {
    type Output = P2;

    fn mul(self, rhs: f32) -> P2 {
        P2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<(f32, f32)> for P2 {
    type Output = P2;

    fn mul(self, pair: (f32, f32)) -> P2 {
        P2 {
            x: self.x * pair.0,
            y: self.y * pair.1,
        }
    }
}

impl Div<f32> for P2 {
    type Output = P2;

    fn div(self, rhs: f32) -> P2 {
        P2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<(f32, f32)> for P2 {
    type Output = P2;

    fn div(self, pair: (f32, f32)) -> P2 {
        P2 {
            x: self.x / pair.0,
            y: self.y / pair.1,
        }
    }
}

impl AddAssign<V2> for P2 {
    fn add_assign(&mut self, rhs: V2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign<V2> for P2 {
    fn sub_assign(&mut self, rhs: V2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl MulAssign<f32> for P2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl MulAssign<(f32, f32)> for P2 {
    fn mul_assign(&mut self, rhs: (f32, f32)) {
        self.x *= rhs.0;
        self.y *= rhs.1;
    }
}

impl DivAssign<f32> for P2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl DivAssign<(f32, f32)> for P2 {
    fn div_assign(&mut self, rhs: (f32, f32)) {
        self.x /= rhs.0;
        self.y /= rhs.1;
    }
}

#[allow(clippy::len_without_is_empty)] // Vector "len" different from list "len"
impl V2 {
    pub fn new(x: f32, y: f32) -> V2 {
        V2 { x, y }
    }

    pub fn zero() -> V2 {
        V2::new(0., 0.)
    }

    /// The squared length of this vector.
    ///
    /// This is occasionally preferable to getting the actual length as it may avoid an unnecessary
    /// sqrt operation. For example, call this if you just want to compare two vectors to see if
    /// one is longer than the other.
    pub fn len2(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// The length of this vector.
    pub fn len(self) -> f32 {
        self.len2().sqrt()
    }

    /// A normalized copy of this vector.
    ///
    /// Note: The zero vector returns itself
    pub fn normalized(self) -> V2 {
        if self == V2::zero() {
            self
        } else {
            self / self.len()
        }
    }
    /// Normalize this vector in place (meaning the vector will point in the same direction but have
    /// a length of 1.0)
    pub fn normalize(&mut self) {
        *self /= self.len();
    }
}

impl Default for V2 {
    fn default() -> Self {
        V2::zero()
    }
}

impl From<(f32, f32)> for V2 {
    fn from(pair: (f32, f32)) -> Self {
        V2 {
            x: pair.0,
            y: pair.1,
        }
    }
}

impl From<P2> for V2 {
    fn from(pt: P2) -> Self {
        pt - P2::zero()
    }
}

impl Add<V2> for V2 {
    type Output = V2;

    fn add(self, rhs: V2) -> V2 {
        V2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<V2> for V2 {
    type Output = V2;

    fn sub(self, rhs: V2) -> V2 {
        V2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for V2 {
    type Output = V2;

    fn mul(self, rhs: f32) -> V2 {
        V2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<(f32, f32)> for V2 {
    type Output = V2;

    fn mul(self, pair: (f32, f32)) -> V2 {
        V2 {
            x: self.x * pair.0,
            y: self.y * pair.1,
        }
    }
}

impl Div<f32> for V2 {
    type Output = V2;

    fn div(self, rhs: f32) -> V2 {
        V2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<(f32, f32)> for V2 {
    type Output = V2;

    fn div(self, pair: (f32, f32)) -> V2 {
        V2 {
            x: self.x / pair.0,
            y: self.y / pair.1,
        }
    }
}

impl AddAssign<V2> for V2 {
    fn add_assign(&mut self, rhs: V2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign<V2> for V2 {
    fn sub_assign(&mut self, rhs: V2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl MulAssign<f32> for V2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl MulAssign<(f32, f32)> for V2 {
    fn mul_assign(&mut self, pair: (f32, f32)) {
        self.x *= pair.0;
        self.y *= pair.1;
    }
}

impl DivAssign<f32> for V2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl DivAssign<(f32, f32)> for V2 {
    fn div_assign(&mut self, pair: (f32, f32)) {
        self.x /= pair.0;
        self.y /= pair.1;
    }
}
