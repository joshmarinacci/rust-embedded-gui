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
            w: x2 - x,
            h: y2 - y,
        }
    }
}

impl Bounds {
    pub fn x2(&self) -> i32 {
        self.x + self.w
    }
    pub fn y2(&self) -> i32 {
        self.y + self.h
    }
    pub fn union(&self, b: Bounds) -> Bounds {
        if self.is_empty() {
            return b;
        }
        if b.is_empty() {
            return *self;
        }
        Bounds::from_xyxy2(
            self.x.min(b.x),
            self.y.min(b.y),
            self.x2().max(b.x2()),
            self.y2().max(b.y2()),
        )
    }
    pub fn center_at(&self, x: i32, y: i32) -> Bounds {
        Bounds {
            x: x - self.w / 2,
            y: y - self.h / 2,
            w: self.w,
            h: self.h,
        }
    }
}

impl Bounds {
    pub fn new_empty() -> Bounds {
        Bounds {
            x: 0,
            y: 0,
            w: -1,
            h: -1,
        }
    }
}

impl Bounds {
    pub fn is_empty(&self) -> bool {
        self.w < 1 || self.h < 1
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
        Bounds { x, y, w, h }
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

mod tests {
    use crate::geom::{Bounds, Point};

    #[test]
    fn test_geometry() {
        let bounds = Bounds {
            x: 0,
            y: 0,
            w: 100,
            h: 100,
        };
        assert_eq!(bounds.contains(&Point::new(10, 10)), true);
        assert_eq!(bounds.contains(&Point::new(-1, -1)), false);

        let b2 = Bounds::new(140, 180, 80, 30);
        let b3 = Bounds::new(140, 180, 80, 30);
        // INFO - union Bounds { x: 140, y: 180, w: 80, h: 30 } Bounds { x: 140, y: 180, w: 80, h: 30 }
        assert_eq!(b2.union(b3), b2.clone());
    }

}