#[derive(Default, Copy, Clone)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy + Clone> Vector2<T> {
    pub fn new(x: T, y: T) -> Vector2<T> {
        Vector2 { x, y }
    }
    pub fn from_scalar(s: T) -> Vector2<T> {
        Vector2 { x: s, y: s }
    }
}

use std::ops;

impl<T: ops::Add<Output = T>> ops::Add<Vector2<T>> for Vector2<T> {
    type Output = Vector2<T>;

    fn add(self, rhs: Vector2<T>) -> Vector2<T> {
        Vector2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub<Vector2<T>> for Vector2<T> {
    type Output = Vector2<T>;

    fn sub(self, rhs: Vector2<T>) -> Vector2<T> {
        Vector2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: ops::Mul<Output = T>> ops::Mul<Vector2<T>> for Vector2<T> {
    type Output = Vector2<T>;

    fn mul(self, rhs: Vector2<T>) -> Vector2<T> {
        Vector2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<T: ops::Div<Output = T>> ops::Div<Vector2<T>> for Vector2<T> {
    type Output = Vector2<T>;

    fn div(self, rhs: Vector2<T>) -> Vector2<T> {
        Vector2 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl<T: ops::Add<Output = T> + Copy> ops::AddAssign<Vector2<T>> for Vector2<T> {
    fn add_assign(&mut self, rhs: Vector2<T>) {
        *self = *self + rhs;
    }
}

impl<T: ops::Sub<Output = T> + Copy> ops::SubAssign<Vector2<T>> for Vector2<T> {
    fn sub_assign(&mut self, rhs: Vector2<T>) {
        *self = *self - rhs;
    }
}

impl<T: ops::Mul<Output = T> + Copy> ops::MulAssign<Vector2<T>> for Vector2<T> {
    fn mul_assign(&mut self, rhs: Vector2<T>) {
        *self = *self * rhs;
    }
}

impl<T: ops::Div<Output = T> + Copy> ops::DivAssign<Vector2<T>> for Vector2<T> {
    fn div_assign(&mut self, rhs: Vector2<T>) {
        *self = *self / rhs;
    }
}
