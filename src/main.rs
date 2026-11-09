use std::fs::File;
use std::io::BufWriter;
use std::process::exit;

use renderer::canvas::{Canvas, write_ppm};
use renderer::demos::{self, Demo};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

const USAGE: &str = "usage: renderer [triangle|cube] [--ppm <file>] [--time <seconds>]";

struct Args {
    demo: Demo,
    ppm: Option<String>,
    time: f32,
}

fn usage(msg: &str) -> ! {
    if !msg.is_empty() {
        eprintln!("error: {msg}");
    }
    eprintln!("{USAGE}");
    exit(1);
}

fn parse_args() -> Args {
    let mut args = Args {
        demo: Demo::Cube,
        ppm: None,
        time: 1.0,
    };

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "triangle" => args.demo = Demo::Triangle,
            "cube" => args.demo = Demo::Cube,
            "--ppm" => {
                i += 1;
                if i >= argv.len() {
                    usage("--ppm needs a file");
                }
                args.ppm = Some(argv[i].clone());
            }
            "--time" => {
                i += 1;
                if i >= argv.len() {
                    usage("--time needs a number");
                }
                match argv[i].parse::<f32>() {
                    Ok(t) => args.time = t,
                    Err(_) => usage("--time needs a number"),
                }
            }
            "-h" | "--help" => usage(""),
            other => usage(&format!("unknown arg {other}")),
        }
        i += 1;
    }
    args
}

fn main() {
    let args = parse_args();
    let mut canvas = Canvas::new(WIDTH, HEIGHT);

    if let Some(path) = args.ppm {
        demos::render(args.demo, &mut canvas, args.time);
        let file = match File::create(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("error: {path}: {e}");
                exit(1);
            }
        };
        if let Err(e) = write_ppm(&canvas, &mut BufWriter::new(file)) {
            eprintln!("error: {path}: {e}");
            exit(1);
        }
        return;
    }

    run_window(args.demo, &mut canvas);
}

#[cfg(not(feature = "x11"))]
fn run_window(_demo: Demo, _canvas: &mut Canvas) {
    eprintln!("built without x11, use --ppm <file>");
    exit(1);
}

#[cfg(feature = "x11")]
fn run_window(demo: Demo, canvas: &mut Canvas) {
    use std::thread::sleep;
    use std::time::{Duration, Instant};
    use window::*;

    const FPS: u64 = 60;

    let width = canvas.width as u32;
    let height = canvas.height as u32;
    let start = Instant::now();

    unsafe {
        let (display, screen, root) = create_window();
        let (window, gc, mut wm_delete) = create_graphics_context(display, root, width, height);
        let image = create_image(display, screen, &mut canvas.pixels, width, height);

        loop {
            if should_quit(display, &mut wm_delete) {
                break;
            }

            demos::render(demo, canvas, start.elapsed().as_secs_f32());

            x11::xlib::XPutImage(display, window, gc, image, 0, 0, 0, 0, width, height);
            x11::xlib::XFlush(display);

            sleep(Duration::from_millis(1000 / FPS));
        }

        destroy_window(display, window, gc);
    }
}

#[cfg(feature = "x11")]
#[allow(non_upper_case_globals)]
mod window {
    use std::ffi::c_char;
    use std::process::exit;
    use std::ptr::{null, null_mut};

    use x11::keysym::{XK_Escape, XK_q};
    use x11::xlib::*;

    pub unsafe fn create_window() -> (*mut Display, i32, Window) {
        unsafe {
            let display = XOpenDisplay(null());
            if display.is_null() {
                eprintln!("error: can't open display, is $DISPLAY set?");
                exit(1);
            }
            let screen = XDefaultScreen(display);
            let root = XRootWindow(display, screen);
            (display, screen, root)
        }
    }

    pub unsafe fn create_graphics_context(
        display: *mut Display,
        root: Window,
        width: u32,
        height: u32,
    ) -> (Window, GC, Atom) {
        unsafe {
            let window = XCreateSimpleWindow(display, root, 0, 0, width, height, 1, 0, 0);
            XStoreName(display, window, c"renderer".as_ptr());
            XSelectInput(display, window, KeyPressMask);

            // so the close button sends a message instead of killing the connection
            let mut wm_delete = XInternAtom(display, c"WM_DELETE_WINDOW".as_ptr(), False);
            XSetWMProtocols(display, window, &mut wm_delete, 1);
            XMapWindow(display, window);

            let gc = XCreateGC(display, window, 0, null_mut());
            (window, gc, wm_delete)
        }
    }

    // image points straight at the pixel vec, so it must not be reallocated or XDestroyImage'd
    pub unsafe fn create_image(
        display: *mut Display,
        screen: i32,
        pixels: &mut Vec<u32>,
        width: u32,
        height: u32,
    ) -> *mut XImage {
        unsafe {
            let image = XCreateImage(
                display,
                XDefaultVisual(display, screen),
                XDefaultDepth(display, screen) as u32,
                ZPixmap,
                0,
                pixels.as_mut_ptr() as *mut c_char,
                width,
                height,
                32,
                0,
            );
            if image.is_null() {
                eprintln!("error: XCreateImage failed");
                exit(1);
            }
            image
        }
    }

    pub unsafe fn should_quit(display: *mut Display, wm_delete: &mut Atom) -> bool {
        unsafe {
            while XPending(display) > 0 {
                let mut event: XEvent = std::mem::zeroed();
                XNextEvent(display, &mut event);
                match event.get_type() {
                    ClientMessage => {
                        if event.client_message.data.get_long(0) as Atom == *wm_delete {
                            return true;
                        }
                    }
                    KeyPress => {
                        let key = XLookupKeysym(&mut event.key, 0);
                        if key == XK_Escape as KeySym || key == XK_q as KeySym {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        }
    }

    pub unsafe fn destroy_window(display: *mut Display, window: Window, gc: GC) {
        unsafe {
            XFreeGC(display, gc);
            XDestroyWindow(display, window);
            XCloseDisplay(display);
        }
    }
}
