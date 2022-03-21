pub mod cache;
pub mod data;
pub mod hint;
pub mod scale;

mod var;

// TODO: move these somewhere more appropriate
#[inline(always)]
pub(crate) fn mul(a: i32, b: i32) -> i32 {
    let ab = a as i64 * b as i64;
    ((ab + 0x8000 - if ab < 0 { 1 } else { 0 }) >> 16) as i32
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
