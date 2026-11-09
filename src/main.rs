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

#[cfg(not(feature = "window"))]
fn run_window(_demo: Demo, _canvas: &mut Canvas) {
    eprintln!("built without a window, use --ppm <file>");
    exit(1);
}

#[cfg(feature = "window")]
fn run_window(demo: Demo, canvas: &mut Canvas) {
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    use renderer::window;

    const FPS: u64 = 60;

    let start = Instant::now();

    unsafe {
        let mut window = window::open_window("renderer", canvas.width as u32, canvas.height as u32);

        while !window::should_quit(&mut window) {
            demos::render(demo, canvas, start.elapsed().as_secs_f32());
            window::present(&window, canvas);
            sleep(Duration::from_millis(1000 / FPS));
        }

        window::close_window(window);
    }
}
