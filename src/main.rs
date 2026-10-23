use std::{thread::sleep, time::Duration};

#[allow(unused)]
use x11::xlib::*;

const FRAMES: usize = 30;

#[derive(Copy, Clone, Debug)]
struct Point {
    x: i16,
    y: i16,
}

#[allow(unused)]
impl Point {
    pub fn new(x: i16, y: i16) -> Point {
        Point {
            x,
            y,
        }
    }

    pub fn mut_point(&mut self, x: i16, y: i16) {
        self.x = x;
        self.y = y;
    }

    pub fn rotate(&mut self, cofr: &Point, angle: f32) {
        let cos_step = f32::cos(angle);
        let sin_step = f32::sin(angle);
        let x = (self.x - cofr.x) as f32;
        let y = (self.y - cofr.y) as f32;

        let new_x = ((x * cos_step - y * sin_step) + cofr.x as f32).round() as i16;
        let new_y = ((x * sin_step + y * cos_step) + cofr.y as f32).round() as i16;
        self.mut_point(new_x, new_y);
    }
}

// todo:

fn main() {
    let width = 800;
    let height = 600;
    let mut pixels: Vec<u32> = vec![0; width * height];
    unsafe {
        let (display, screen, root) = create_window();
        let (window, gc) = create_graphics_context(display, root, width, height);
        let image = create_image(display, screen, &mut pixels, width as u32, height as u32);

        let d1 = Point::new(400, 250);
        let d2 = Point::new(350, 350);
        let d3 = Point::new(450, 350);

        loop {
            pixels.fill(0xaabbcc); // background

            draw_triangle([d1, d2, d3], width, height, &mut pixels, 0xff0000);

            XPutImage(display, window, gc, image, 0, 0, 0, 0, width as u32, height as u32);
            XFlush(display);

            sleep(Duration::from_millis((1000 / FRAMES) as u64));
        }
    }
}

fn set_pixel(x: i16, y: i16, width: usize, height: usize, pixels: &mut [u32], color: u32) {
    if x < 0 || x >= width as i16 || y < 0 || y >= height as i16 {
        return;
    }

    let idx = (y as i32 * width as i32 + x as i32) as usize;
    pixels[idx] = color;
}

fn angle(p1: &Point, center: &Point, p3: &Point) -> f32 {
    let v1 = ((p1.x - center.x) as f32, (p1.y - center.y) as f32);
    let v3 = ((p3.x - center.x) as f32, (p3.y - center.y) as f32);

    let mag1 = (v1.0 * v1.0 + v1.1 * v1.1).sqrt();
    let mag3 = (v3.0 * v3.0 + v3.1 * v3.1).sqrt();
    let dot = v1.0 * v3.0 + v1.1 * v3.1;

    let cos_angle = (dot / (mag1 * mag3)).clamp(-1.0, 1.0);

    cos_angle.acos()
}

fn centroid(p1: &Point, p2: &Point, p3: &Point) -> Point {
    return Point::new((p1.x + p2.x + p3.x) / 3, (p1.y + p2.y + p3.y) / 3); 
}

fn draw_triangle(
    points: [Point;3],
    width: usize,
    height: usize,
    pixels: &mut [u32],
    color: u32,
) {
    let p0 = points[0];
    let p1 = points[1];
    let p2 = points[2];
    let min_x = p0.x.min(p1.x).min(p2.x);
    let max_x = p0.x.max(p1.x).max(p2.x);
    let min_y = p0.y.min(p1.y).min(p2.y);
    let max_y = p0.y.max(p1.y).max(p2.y);

    let area = edge_function(&p0, &p1, &p2);
    if area == 0 { return; }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Point::new(x,y);

            let w0 = edge_function(&p1, &p2, &p);
            let w1 = edge_function(&p2, &p0, &p);
            let w2 = edge_function(&p0, &p1, &p);

            let inside = 
            (w0 >= 0 && w1 >= 0 && w2 >= 0) 
            ||
            (w0 <= 0 && w1 <= 0 && w2 <= 0);

            if inside {
                set_pixel(x, y, width, height, pixels, color);
            }
        }
    }
}

fn draw_circle(width: usize, height: usize, pixels: &mut Vec<u32>, radius: i32, cx: i32, cy: i32, color: u32) {
    let r2 = radius * radius;

    let min_x = (cx - radius).max(0);
    let max_x = (cx + radius).min(width as i32 - 1);
    let min_y = (cy - radius).max(0);
    let max_y = (cy + radius).min(height as i32 - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x - cx;
            let dy = y - cy;

            if dx * dx + dy * dy <= r2 {
                set_pixel(x as i16, y as i16, width, height, pixels, color);
            }
        }
    }
}

fn edge_function(a: &Point, b: &Point, c: &Point) -> i16 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}

fn draw_line(
    width: usize,
    height: usize,
    pixels: &mut Vec<u32>,
    (x1, y1): (i16, i16),
    (x2, y2): (i16, i16),
    color: u32,
) {
    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;

    let mut x = x1;
    let mut y = y1;

    loop {
        if x >= 0 && x < width as i16 && y >= 0 && y < height as i16 {
            set_pixel(x, y, width, height, pixels, color);
        }

        if x == x2 && y == y2 {
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

fn draw_checker(width: usize, height: usize, pixels: &mut Vec<u32>) {
    for i in 0..width*height {
        if (i % 2) == 0 {
            pixels[i] = 0xff0000;
            continue
        }
        pixels[i] = 0x0000ff;
    }
}

unsafe fn create_window() -> (*mut _XDisplay, i32, u64) {
    unsafe {
        let display = XOpenDisplay(std::ptr::null());

        if display.is_null() {
            panic!("Cannot open X display");
        }

        let screen = XDefaultScreen(display);
        let root = XRootWindow(display, screen);
        (display, screen, root)
    }
}

unsafe fn create_graphics_context(display: *mut _XDisplay, root: u64, width: usize, height: usize) -> (u64, *mut _XGC) {
    unsafe {
        let window = XCreateSimpleWindow(
            display,
            root,
            0,
            0,
            width as u32,
            height as u32,
            1,
            0,
            0,
        );

        XMapWindow(display, window);
        XFlush(display);

        (window, XCreateGC(display, window, 0, std::ptr::null_mut()))
    }
}

unsafe fn create_image(display: *mut _XDisplay, screen: i32, pixels: &mut Vec<u32>, width: u32, height: u32) -> *mut XImage {
    unsafe {
        let visual = XDefaultVisual(display, screen);
        let depth = XDefaultDepth(display, screen);

        XCreateImage(
            display,
            visual,
            depth as u32,
            ZPixmap,
            0,
            pixels.as_mut_ptr() as *mut i8,
            width,
            height,
            32,
            0,
        )
    }
}
