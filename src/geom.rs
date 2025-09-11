#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Bounds {
    fn from_xyxy2(x: i32, y: i32, x2: i32, y2: i32) -> Bounds {
        Bounds {
            x,
            y,
            w: x + x2,
            h: y + y2,
        }
    }
}

impl Bounds {
    pub fn x2(&self) -> i32 { self.x + self.w }
    pub fn y2(&self) -> i32 { self.y + self.h }
    pub(crate) fn union(&self, p0: Bounds) -> Bounds {
        if self.is_empty() {
            return p0.clone();
        }
        if p0.is_empty() {
            return self.clone();
        }
        Bounds::from_xyxy2(
            self.x.min(p0.x),
            self.y.min(p0.y),
            self.x2().max(p0.x2()),
            self.y2().max(p0.y2()),
        )
    }
}

impl Bounds {
    pub fn new_empty() -> Bounds {
        Bounds {
            x: 0,
            y:0,
            w:-1,
            h:-1
        }
    }
}

impl Bounds {
    pub fn is_empty(&self) -> bool {
        self.w < 1 || self.h < 0
    }
    pub(crate) fn contract(&self, amt: i32) -> Bounds {
        Bounds {
            x: self.x + amt,
            y: self.y + amt,
            w: self.w - amt - amt,
            h: self.h - amt - amt,
        }
    }
}

impl Bounds {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Bounds {
        Bounds {
            x,
            y,
            w,
            h,
        }
    }
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