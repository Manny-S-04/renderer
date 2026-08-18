use std::{thread::sleep, time::Duration};

#[allow(unused)]
use x11::xlib::*;

const FRAMES: i32 = 33;

#[derive(Copy, Clone, Debug)]
struct Point {
    x: i16,
    y: i16,
    positions: [(i16, i16); FRAMES as usize],
    ptr: usize
}

impl Default for Point {
    fn default() -> Self {
        Self { x: Default::default(), y: Default::default(), positions: [(0, 0); FRAMES as usize], ptr: Default::default() }
    }
}


#[allow(unused)]
impl Point {
    pub fn new(x: i16, y: i16) -> Point { 
        Point {
            x,
            y,
            positions: [(0 as i16, 0 as i16); FRAMES as usize],
            ptr: 0,
        }
    }

    pub fn mut_point(&mut self, x: i16, y: i16) {
        self.x = x;
        self.y = y;
    }

    pub fn rotate(&mut self, center: &Point, destination: &Point, infinite: bool) {
        if self.ptr == 0 {
            self.calc_frames(center, destination);
        }
        if self.ptr < FRAMES as usize {
            let pos = self.positions[self.ptr];
            self.mut_point(pos.0, pos.1);
            self.ptr += 1;
            if infinite && self.ptr >= FRAMES as usize{ 
                self.ptr = 0;
            }
        }
    }

    fn calc_frames(&mut self, center: &Point, destination: &Point) {
        // subtract off center?
        let max_angle = angle(&self, &center, &destination);
        let step = max_angle / FRAMES as f32;
        let mut x = self.x as f32 - center.x as f32;
        let mut y = self.y as f32 - center.y as f32;
        let cos_step = f32::cos(step);
        let sin_step = f32::sin(step);

        for i in 0..FRAMES as usize{
            let new_x = x * cos_step - y * sin_step;
            let new_y = x * sin_step + y * cos_step;

            self.positions[i] = 
                ((new_x + center.x as f32).round() as i16,
                    (new_y + center.y as f32).round() as i16);

            x = new_x;
            y = new_y;
        }
    }
}


// todo:
// think of how to redesign the api so its more ergonomic
// draw multiple triangles
// add a gradient
// more shapes?
// learn how to make a background persist
// learn to draw an animated object on the background
// user input, maybe here pause the screen no updates until user input
// -> exit out of immediate mode

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

        let mut p1 = d1.clone();
        let mut p2 = d2.clone();
        let mut p3 = d3.clone();

        let mut center = centroid(&p1, &p2, &p3);
        loop {
            pixels.fill(0xaabbcc);

            //draw_circle(width, height, &mut pixels, 5, p1.x as i32, p1.y as i32, 0xffffff);
            //draw_circle(width, height, &mut pixels, 5, p2.x as i32, p2.y as i32, 0xffffff);
            //draw_circle(width, height, &mut pixels, 5, p3.x as i32, p3.y as i32, 0xffffff);
            draw_triangle(width as i16, height as i16, &mut pixels, &p1, &p2, &p3, 0xffffff);
            draw_circle(width as i16, height as i16, &mut pixels, 5, center.x as i32, center.y as i32, 0xff0000);

            XPutImage(display, window, gc, image, 0, 0, 0, 0, width as u32, height as u32);
            XFlush(display);

            // i want a triangle object which captures these 3 points
            // so i can do triangle.rotate(&center);
            p1.rotate(&center, &p3, true);
            p3.rotate(&center, &p2, true);
            p2.rotate(&center, &p1, true);

            sleep(Duration::from_millis((1 / FRAMES) as u64));
        }
    }
}

fn set_pixel(x: i16, y: i16, width: i16, height: i16, pixels: &mut [u32], color: u32) {
    if x < 0 || x >= width || y < 0 || y >= height {
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
/*

fn calc_step(pi: &Point, pf: &Point) -> (i16, i16) {
    let dx = pf.x - pi.x;
    let dy = pf.y - pi.y;

    let step_x = dx / FRAMES as i16;
    let step_y = dy / FRAMES as i16;

    return (step_x, step_y);
}
*/

fn draw_circle(width: i16, height: i16, pixels: &mut Vec<u32>, radius: i32, cx: i32, cy: i32, color: u32) {
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

fn draw_triangle(
    width: i16,
    height: i16,
    pixels: &mut [u32],
    p0: &Point,
    p1: &Point,
    p2: &Point,
    color: u32,
) {
    let min_x = p0.x.min(p1.x).min(p2.x);
    let max_x = p0.x.max(p1.x).max(p2.x);
    let min_y = p0.y.min(p1.y).min(p2.y);
    let max_y = p0.y.max(p1.y).max(p2.y);

    let area = edge_function(&p0, &p1, &p2);
    if area == 0 { return; }

    // do creation once and iterate over
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

fn draw_line(
    width: i16,
    height: i16,
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
        if x >= 0 && x < width && y >= 0 && y < height {
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

#[allow(unused_variables)]
#[allow(unused)]
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
