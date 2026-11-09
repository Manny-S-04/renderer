use std::io::{self, Write};

use crate::color::{self, Color};
use crate::math::Vec2;

pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        Canvas {
            width,
            height,
            pixels: vec![color::BLACK; width * height],
        }
    }
}

fn index(canvas: &Canvas, x: i32, y: i32) -> Option<usize> {
    if x < 0 || y < 0 || x as usize >= canvas.width || y as usize >= canvas.height {
        return None;
    }
    Some(y as usize * canvas.width + x as usize)
}

pub fn get_pixel(canvas: &Canvas, x: i32, y: i32) -> Option<Color> {
    index(canvas, x, y).map(|i| canvas.pixels[i])
}

pub fn set_pixel(canvas: &mut Canvas, x: i32, y: i32, c: Color) {
    if let Some(i) = index(canvas, x, y) {
        canvas.pixels[i] = color::blend(canvas.pixels[i], c);
    }
}

pub fn fill(canvas: &mut Canvas, c: Color) {
    canvas.pixels.fill(c);
}

pub fn fill_rect(canvas: &mut Canvas, x: i32, y: i32, w: i32, h: i32, c: Color) {
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + w).min(canvas.width as i32);
    let y1 = (y + h).min(canvas.height as i32);
    for py in y0..y1 {
        for px in x0..x1 {
            set_pixel(canvas, px, py, c);
        }
    }
}

pub fn fill_circle(canvas: &mut Canvas, cx: i32, cy: i32, radius: i32, c: Color) {
    let r2 = radius as i64 * radius as i64;

    let min_x = (cx - radius).max(0);
    let max_x = (cx + radius).min(canvas.width as i32 - 1);
    let min_y = (cy - radius).max(0);
    let max_y = (cy + radius).min(canvas.height as i32 - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = (x - cx) as i64;
            let dy = (y - cy) as i64;
            if dx * dx + dy * dy <= r2 {
                set_pixel(canvas, x, y, c);
            }
        }
    }
}

// bresenham
pub fn draw_line(canvas: &mut Canvas, x0: i32, y0: i32, x1: i32, y1: i32, c: Color) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        set_pixel(canvas, x, y, c);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

pub fn fill_triangle(canvas: &mut Canvas, points: [Vec2; 3], c: Color) {
    rasterize(canvas, points, |_| c);
}

pub fn fill_triangle_gradient(canvas: &mut Canvas, points: [Vec2; 3], colors: [Color; 3]) {
    rasterize(canvas, points, |w| color::mix3(colors, w));
}

fn edge_function(a: Vec2, b: Vec2, p: Vec2) -> f32 {
    (b - a).cross(p - a)
}

// top-left rule so shared edges only get drawn once
fn is_top_left(a: Vec2, b: Vec2) -> bool {
    let d = b - a;
    (d.y == 0.0 && d.x > 0.0) || d.y < 0.0
}

fn inside(w: f32, owns_edge: bool) -> bool {
    w > 0.0 || (w == 0.0 && owns_edge)
}

fn rasterize(canvas: &mut Canvas, points: [Vec2; 3], shader: impl Fn([f32; 3]) -> Color) {
    let p0 = points[0];
    let mut p1 = points[1];
    let mut p2 = points[2];

    let mut area = edge_function(p0, p1, p2);
    if area == 0.0 || !area.is_finite() {
        return;
    }
    if area < 0.0 {
        std::mem::swap(&mut p1, &mut p2);
        area = -area;
    }

    let min_x = (p0.x.min(p1.x).min(p2.x).floor() as i32).max(0);
    let min_y = (p0.y.min(p1.y).min(p2.y).floor() as i32).max(0);
    let max_x = (p0.x.max(p1.x).max(p2.x).ceil() as i32).min(canvas.width as i32 - 1);
    let max_y = (p0.y.max(p1.y).max(p2.y).ceil() as i32).min(canvas.height as i32 - 1);

    let owns0 = is_top_left(p1, p2);
    let owns1 = is_top_left(p2, p0);
    let owns2 = is_top_left(p0, p1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);

            let w0 = edge_function(p1, p2, p);
            let w1 = edge_function(p2, p0, p);
            let w2 = edge_function(p0, p1, p);

            if inside(w0, owns0) && inside(w1, owns1) && inside(w2, owns2) {
                set_pixel(canvas, x, y, shader([w0 / area, w1 / area, w2 / area]));
            }
        }
    }
}

pub fn write_ppm(canvas: &Canvas, out: &mut impl Write) -> io::Result<()> {
    write!(out, "P6\n{} {}\n255\n", canvas.width, canvas.height)?;
    let mut bytes: Vec<u8> = Vec::with_capacity(canvas.pixels.len() * 3);
    for c in &canvas.pixels {
        let [r, g, b, _] = color::channels(*c);
        bytes.push(r);
        bytes.push(g);
        bytes.push(b);
    }
    out.write_all(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{BLACK, BLUE, GREEN, RED, WHITE, rgba};

    fn count(canvas: &Canvas, c: Color) -> usize {
        canvas.pixels.iter().filter(|p| **p == c).count()
    }

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    #[test]
    fn clips_off_canvas() {
        let mut c = Canvas::new(32, 32);
        fill_rect(&mut c, -100, -100, 1000, 1000, WHITE);
        fill_circle(&mut c, -5, 40, 20, RED);
        draw_line(&mut c, -50, -50, 100, 80, BLUE);
        fill_triangle(&mut c, [v(-1e4, -1e4), v(1e4, 0.0), v(0.0, 1e4)], GREEN);
    }

    #[test]
    fn big_triangle() {
        let mut c = Canvas::new(800, 600);
        fill_triangle(&mut c, [v(0.0, 0.0), v(800.0, 0.0), v(0.0, 600.0)], WHITE);
        assert_eq!(count(&c, WHITE), 800 * 600 / 2);
    }

    #[test]
    fn winding_order() {
        let tri = [v(3.0, 2.0), v(20.0, 5.0), v(8.0, 18.0)];
        let mut cw = Canvas::new(24, 24);
        let mut ccw = Canvas::new(24, 24);
        fill_triangle(&mut cw, tri, WHITE);
        fill_triangle(&mut ccw, [tri[0], tri[2], tri[1]], WHITE);
        assert!(count(&cw, WHITE) > 0);
        assert_eq!(cw.pixels, ccw.pixels);
    }

    #[test]
    fn shared_edge_drawn_once() {
        let half = rgba(255, 255, 255, 128);
        let mut c = Canvas::new(16, 16);
        let a = v(2.0, 2.0);
        let b = v(14.0, 2.0);
        let d = v(14.0, 14.0);
        let e = v(2.0, 14.0);
        fill_triangle(&mut c, [a, b, d], half);
        fill_triangle(&mut c, [a, d, e], half);
        assert_eq!(count(&c, color::blend(BLACK, half)), 12 * 12);
        assert_eq!(count(&c, BLACK), 16 * 16 - 12 * 12);
    }

    #[test]
    fn degenerate_triangle() {
        let mut c = Canvas::new(8, 8);
        fill_triangle(&mut c, [v(1.0, 1.0), v(4.0, 4.0), v(7.0, 7.0)], WHITE);
        fill_triangle(&mut c, [v(f32::NAN, 1.0), v(4.0, 4.0), v(7.0, 1.0)], WHITE);
        assert_eq!(count(&c, WHITE), 0);
    }

    #[test]
    fn gradient_corner() {
        let mut c = Canvas::new(101, 101);
        fill_triangle_gradient(
            &mut c,
            [v(0.0, 0.0), v(100.0, 0.0), v(0.0, 100.0)],
            [RED, GREEN, BLUE],
        );
        let [r, g, b, _] = color::channels(get_pixel(&c, 0, 0).unwrap());
        assert!(r > 245 && g < 10 && b < 10);
    }

    #[test]
    fn line_endpoints() {
        let mut c = Canvas::new(20, 20);
        draw_line(&mut c, 1, 2, 18, 9, WHITE);
        assert_eq!(get_pixel(&c, 1, 2), Some(WHITE));
        assert_eq!(get_pixel(&c, 18, 9), Some(WHITE));
        assert_eq!(count(&c, WHITE), 18);
    }

    #[test]
    fn circle_symmetric() {
        let mut c = Canvas::new(41, 41);
        fill_circle(&mut c, 20, 20, 10, WHITE);
        for y in 0..41 {
            for x in 0..41 {
                assert_eq!(get_pixel(&c, x, y), get_pixel(&c, 40 - x, y));
                assert_eq!(get_pixel(&c, x, y), get_pixel(&c, y, x));
            }
        }
    }

    #[test]
    fn ppm() {
        let c = Canvas::new(3, 2);
        let mut out: Vec<u8> = Vec::new();
        write_ppm(&c, &mut out).unwrap();
        assert!(out.starts_with(b"P6\n3 2\n255\n"));
        assert_eq!(out.len(), "P6\n3 2\n255\n".len() + 3 * 2 * 3);
    }
}
