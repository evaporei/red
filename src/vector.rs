#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
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

#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub struct Vector4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T: Copy + Clone> Vector4<T> {
    pub fn new(x: T, y: T, z: T, w: T) -> Vector4<T> {
        Vector4 { x, y, z, w }
    }
    pub fn from_scalar(s: T) -> Vector4<T> {
        Vector4 {
            x: s,
            y: s,
            z: s,
            w: s,
        }
    }
}

impl<T: ops::Add<Output = T>> ops::Add<Vector4<T>> for Vector4<T> {
    type Output = Vector4<T>;

    fn add(self, rhs: Vector4<T>) -> Vector4<T> {
        Vector4 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub<Vector4<T>> for Vector4<T> {
    type Output = Vector4<T>;

    fn sub(self, rhs: Vector4<T>) -> Vector4<T> {
        Vector4 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl<T: ops::Mul<Output = T>> ops::Mul<Vector4<T>> for Vector4<T> {
    type Output = Vector4<T>;

    fn mul(self, rhs: Vector4<T>) -> Vector4<T> {
        Vector4 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            w: self.w * rhs.w,
        }
    }
}

impl<T: ops::Div<Output = T>> ops::Div<Vector4<T>> for Vector4<T> {
    type Output = Vector4<T>;

    fn div(self, rhs: Vector4<T>) -> Vector4<T> {
        Vector4 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
            w: self.w / rhs.w,
        }
    }
}

impl<T: ops::Add<Output = T> + Copy> ops::AddAssign<Vector4<T>> for Vector4<T> {
    fn add_assign(&mut self, rhs: Vector4<T>) {
        *self = *self + rhs;
    }
}

impl<T: ops::Sub<Output = T> + Copy> ops::SubAssign<Vector4<T>> for Vector4<T> {
    fn sub_assign(&mut self, rhs: Vector4<T>) {
        *self = *self - rhs;
    }
}

impl<T: ops::Mul<Output = T> + Copy> ops::MulAssign<Vector4<T>> for Vector4<T> {
    fn mul_assign(&mut self, rhs: Vector4<T>) {
        *self = *self * rhs;
    }
}

impl<T: ops::Div<Output = T> + Copy> ops::DivAssign<Vector4<T>> for Vector4<T> {
    fn div_assign(&mut self, rhs: Vector4<T>) {
        *self = *self / rhs;
    }
}
