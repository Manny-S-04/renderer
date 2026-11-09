use std::ops::{Add, Mul, Sub};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }

    pub fn cross(self, o: Vec2) -> f32 {
        self.x * o.y - self.y * o.x
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    // y is down so positive angle = clockwise
    pub fn rotate(self, cofr: Vec2, angle: f32) -> Vec2 {
        let cos = angle.cos();
        let sin = angle.sin();
        let x = self.x - cofr.x;
        let y = self.y - cofr.y;
        Vec2::new(x * cos - y * sin + cofr.x, x * sin + y * cos + cofr.y)
    }
}

pub fn centroid(points: &[Vec2]) -> Vec2 {
    let mut sum = Vec2::default();
    for p in points {
        sum = sum + *p;
    }
    sum * (1.0 / points.len() as f32)
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, k: f32) -> Vec2 {
        Vec2::new(self.x * k, self.y * k)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn dot(self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len == 0.0 {
            return self;
        }
        self * (1.0 / len)
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, k: f32) -> Vec3 {
        Vec3::new(self.x * k, self.y * k, self.z * k)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mat3(pub [[f32; 3]; 3]);

impl Mat3 {
    pub fn rotation_x(angle: f32) -> Mat3 {
        let (s, c) = angle.sin_cos();
        Mat3([[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]])
    }

    pub fn rotation_y(angle: f32) -> Mat3 {
        let (s, c) = angle.sin_cos();
        Mat3([[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]])
    }

    pub fn rotation_z(angle: f32) -> Mat3 {
        let (s, c) = angle.sin_cos();
        Mat3([[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]])
    }

    pub fn apply(self, v: Vec3) -> Vec3 {
        let m = self.0;
        Vec3::new(
            m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z,
            m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z,
            m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z,
        )
    }
}

// a * b applies b first
impl Mul for Mat3 {
    type Output = Mat3;
    fn mul(self, o: Mat3) -> Mat3 {
        let mut out = [[0.0; 3]; 3];
        for r in 0..3 {
            for c in 0..3 {
                for k in 0..3 {
                    out[r][c] += self.0[r][k] * o.0[k][c];
                }
            }
        }
        Mat3(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec2, b: Vec2) -> bool {
        (a - b).length() < 1e-4
    }

    #[test]
    fn rotate_quarter_turn() {
        let p = Vec2::new(1.0, 0.0).rotate(Vec2::default(), std::f32::consts::FRAC_PI_2);
        assert!(close(p, Vec2::new(0.0, 1.0)));
    }

    #[test]
    fn rotate_keeps_distance() {
        let cofr = Vec2::new(400.0, 300.0);
        let p = Vec2::new(450.0, 350.0);
        for i in 0..100 {
            let r = p.rotate(cofr, i as f32 * 0.37);
            assert!(((r - cofr).length() - (p - cofr).length()).abs() < 1e-3);
        }
        assert!(close(cofr.rotate(cofr, 1.0), cofr));
    }

    #[test]
    fn centroid_of_triangle() {
        let c = centroid(&[
            Vec2::new(0.0, 0.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(0.0, 3.0),
        ]);
        assert!(close(c, Vec2::new(1.0, 1.0)));
    }

    #[test]
    fn rotations_compose() {
        let a = Mat3::rotation_z(0.3) * Mat3::rotation_z(0.4);
        let b = Mat3::rotation_z(0.7);
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert!((a.apply(v) - b.apply(v)).length() < 1e-5);
        assert!((Mat3::rotation_y(1.1).apply(v).length() - v.length()).abs() < 1e-5);
    }

    #[test]
    fn cross_right_hand() {
        let z = Vec3::new(1.0, 0.0, 0.0).cross(Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(z, Vec3::new(0.0, 0.0, 1.0));
    }
}
