use std::f32::consts::TAU;

use crate::canvas::*;
use crate::color::{self, Color, rgba};
use crate::math::{Mat3, Vec2, Vec3, centroid};

pub const BACKGROUND: Color = 0xFF181818;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Demo {
    Triangle,
    Cube,
}

pub fn render(demo: Demo, canvas: &mut Canvas, t: f32) {
    match demo {
        Demo::Triangle => triangle(canvas, t),
        Demo::Cube => cube(canvas, t),
    }
}

pub fn triangle(canvas: &mut Canvas, t: f32) {
    fill(canvas, BACKGROUND);

    let center = Vec2::new(canvas.width as f32 / 2.0, canvas.height as f32 / 2.0);
    let radius = canvas.height as f32 * 0.38;

    let mut points = [Vec2::default(); 3];
    for i in 0..3 {
        let a = -TAU / 4.0 + i as f32 * TAU / 3.0;
        points[i] = center + Vec2::new(a.cos(), a.sin()) * radius;
    }

    // rotate from the original points every frame, not the last frame's, or it drifts
    let cofr = centroid(&points);
    for p in points.iter_mut() {
        *p = p.rotate(cofr, t * 0.8);
    }
    fill_triangle_gradient(canvas, points, [color::RED, color::GREEN, color::BLUE]);

    let half = radius * 0.55;
    let mut square = [
        Vec2::new(center.x - half, center.y - half),
        Vec2::new(center.x + half, center.y - half),
        Vec2::new(center.x + half, center.y + half),
        Vec2::new(center.x - half, center.y + half),
    ];
    for p in square.iter_mut() {
        *p = p.rotate(center, -t * 0.5);
    }

    let glass = rgba(255, 255, 255, 0x50);
    fill_triangle(canvas, [square[0], square[1], square[2]], glass);
    fill_triangle(canvas, [square[0], square[2], square[3]], glass);
}

const CORNERS: [Vec3; 8] = [
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(-1.0, 1.0, 1.0),
    Vec3::new(1.0, 1.0, 1.0),
];

struct Face {
    quad: [usize; 4],
    normal: Vec3,
    color: Color,
}

const FACES: [Face; 6] = [
    Face {
        quad: [0, 2, 6, 4],
        normal: Vec3::new(-1.0, 0.0, 0.0),
        color: 0xFFE06C75,
    },
    Face {
        quad: [1, 3, 7, 5],
        normal: Vec3::new(1.0, 0.0, 0.0),
        color: 0xFF98C379,
    },
    Face {
        quad: [0, 1, 5, 4],
        normal: Vec3::new(0.0, -1.0, 0.0),
        color: 0xFFE5C07B,
    },
    Face {
        quad: [2, 3, 7, 6],
        normal: Vec3::new(0.0, 1.0, 0.0),
        color: 0xFF61AFEF,
    },
    Face {
        quad: [0, 1, 3, 2],
        normal: Vec3::new(0.0, 0.0, -1.0),
        color: 0xFFC678DD,
    },
    Face {
        quad: [4, 5, 7, 6],
        normal: Vec3::new(0.0, 0.0, 1.0),
        color: 0xFF56B6C2,
    },
];

const CAMERA: Vec3 = Vec3::new(0.0, 0.0, 5.0);

pub fn cube_rotation(t: f32) -> Mat3 {
    Mat3::rotation_y(t * 0.9) * Mat3::rotation_x(t * 0.6) * Mat3::rotation_z(t * 0.2)
}

fn project(v: Vec3, width: f32, height: f32) -> Vec2 {
    let focal = height * 0.95;
    Vec2::new(
        width / 2.0 + v.x / v.z * focal,
        height / 2.0 - v.y / v.z * focal,
    )
}

pub fn is_facing_camera(normal: Vec3, rotation: Mat3) -> bool {
    let n = rotation.apply(normal);
    n.dot(n + CAMERA) < 0.0
}

pub fn cube(canvas: &mut Canvas, t: f32) {
    fill(canvas, BACKGROUND);

    let width = canvas.width as f32;
    let height = canvas.height as f32;
    let rotation = cube_rotation(t);
    let light = Vec3::new(-0.4, 0.7, -1.0).normalize();

    let mut world = [Vec3::default(); 8];
    for i in 0..8 {
        world[i] = rotation.apply(CORNERS[i]) + CAMERA;
    }

    // convex so back-face culling is enough, no depth buffer yet
    for face in FACES.iter() {
        if !is_facing_camera(face.normal, rotation) {
            continue;
        }

        let n = rotation.apply(face.normal);
        let brightness = 0.25 + 0.75 * n.dot(light).max(0.0);
        let c = color::shade(face.color, brightness);

        let mut p = [Vec2::default(); 4];
        for k in 0..4 {
            p[k] = project(world[face.quad[k]], width, height);
        }

        fill_triangle(canvas, [p[0], p[1], p[2]], c);
        fill_triangle(canvas, [p[0], p[2], p[3]], c);

        for k in 0..4 {
            let a = p[k];
            let b = p[(k + 1) % 4];
            draw_line(
                canvas,
                a.x.round() as i32,
                a.y.round() as i32,
                b.x.round() as i32,
                b.y.round() as i32,
                0xFF101010,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_frames() {
        for demo in [Demo::Triangle, Demo::Cube] {
            let mut a = Canvas::new(160, 120);
            let mut b = Canvas::new(160, 120);
            render(demo, &mut a, 1.25);
            render(demo, &mut b, 1.25);
            assert_eq!(a.pixels, b.pixels);
            assert!(
                a.pixels.iter().any(|p| *p != BACKGROUND),
                "{demo:?} drew nothing"
            );
        }
    }

    #[test]
    fn cube_visible_faces() {
        for step in 0..50 {
            let t = step as f32 * 0.13;
            let rotation = cube_rotation(t);
            let mut visible = 0;
            for face in FACES.iter() {
                if is_facing_camera(face.normal, rotation) {
                    visible += 1;
                }
            }
            assert!((1..=3).contains(&visible), "t={t}: {visible} faces");
        }
    }
}
