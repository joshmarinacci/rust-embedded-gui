#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Bounds {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) w: i32,
    pub(crate) h: i32,
}

#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
}

impl Bounds {
    pub fn contains(&self, pt: &Point) -> bool {
        if self.x <= pt.x && self.y <= pt.y {
            if self.x + self.w > pt.x && self.y + self.h > pt.y {
                return true;
            }
        }
        false
    }
}