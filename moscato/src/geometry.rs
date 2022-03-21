#[derive(Copy, Clone, Default, Debug)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub xx: f32,
    pub yx: f32,
    pub xy: f32,
    pub yy: f32,
    pub dx: f32,
    pub dy: f32,
}

impl Transform {
    pub fn new(elements: &[f32; 6]) -> Self {
        Self {
            xx: elements[0],
            yx: elements[1],
            xy: elements[2],
            yy: elements[3],
            dx: elements[4],
            dy: elements[5],
        }
    }
    pub fn scale(x: f32, y: f32) -> Self {
        Self::new(&[x, 0., 0., y, 0., 0.])
    }

    pub fn translate(x: f32, y: f32) -> Self {
        Self::new(&[1., 0., 0., 1., x, y])
    }

    pub fn rotate(th: f32) -> Self {
        let (s, c) = th.sin_cos();
        Self::new(&[c, s, -s, c, 0., 0.])
    }

    pub fn skew(x: f32, y: f32) -> Self {
        Self::new(&[1., x.tan(), y.tan(), 1., 0., 0.])
    }

    pub fn around_center(&self, x: f32, y: f32) -> Self {
        Self::translate(x, y) * *self * Self::translate(-x, -y)
    }

    pub fn transform_point(&self, point: &Point) -> Point {
        Point {
            x: point.x * self.xx + point.y * self.yx + self.dx,
            y: point.y * self.yy + point.y * self.xy + self.dy,
        }
    }
}

impl std::ops::Mul for Transform {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(&[
            self.xx * other.xx + self.xy * other.yx,
            self.yx * other.xx + self.yy * other.yx,
            self.xx * other.xy + self.xy * other.yy,
            self.yx * other.xy + self.yy * other.yy,
            self.xx * other.dx + self.xy * other.dy + self.dx,
            self.yx * other.dx + self.yy * other.dy + self.dy,
        ])
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Bounds {
    pub min: Point,
    pub max: Point,
}

impl Bounds {
    pub fn from_points(points: &[Point]) -> Self {
        if points.is_empty() {
            Self::default()
        } else {
            let mut bounds = Self {
                min: Point::new(f32::MAX, f32::MAX),
                max: Point::new(f32::MIN, f32::MIN),
            };
            for point in points {
                bounds.add(point);
            }
            bounds
        }
    }

    pub fn add(&mut self, point: &Point) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
    }

    pub fn transform(&self, transform: &Transform) -> Self {
        Self {
            min: transform.transform_point(&self.min),
            max: transform.transform_point(&self.max),
        }
    }
}
