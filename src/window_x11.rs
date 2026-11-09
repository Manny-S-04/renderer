#![allow(clippy::missing_safety_doc)]
#![allow(non_upper_case_globals)]

// safety: one Window per thread, call everything from the thread that opened it

use std::ffi::{CString, c_char};
use std::process::exit;
use std::ptr::{null, null_mut};

use x11::keysym::*;
use x11::xlib::*;

use crate::canvas::Canvas;
use crate::event::{Event, Key, is_quit};

pub struct Window {
    display: *mut Display,
    window: u64,
    gc: GC,
    image: *mut XImage,
    wm_delete: Atom,
    width: u32,
    height: u32,
}

pub unsafe fn open_window(title: &str, width: u32, height: u32) -> Window {
    unsafe {
        let display = XOpenDisplay(null());
        if display.is_null() {
            eprintln!("error: can't open display, is $DISPLAY set?");
            exit(1);
        }
        let screen = XDefaultScreen(display);
        let root = XRootWindow(display, screen);

        let window = XCreateSimpleWindow(display, root, 0, 0, width, height, 1, 0, 0);
        let title = CString::new(title).unwrap_or_default();
        XStoreName(display, window, title.as_ptr());
        XSelectInput(
            display,
            window,
            KeyPressMask | ButtonPressMask | ButtonReleaseMask | PointerMotionMask,
        );

        // fixed size, the canvas doesn't scale
        let hints = XAllocSizeHints();
        (*hints).flags = PMinSize | PMaxSize;
        (*hints).min_width = width as i32;
        (*hints).max_width = width as i32;
        (*hints).min_height = height as i32;
        (*hints).max_height = height as i32;
        XSetWMNormalHints(display, window, hints);
        XFree(hints.cast());

        // so the close button sends a message instead of killing the connection
        let mut wm_delete = XInternAtom(display, c"WM_DELETE_WINDOW".as_ptr(), False);
        XSetWMProtocols(display, window, &mut wm_delete, 1);
        XMapWindow(display, window);

        let gc = XCreateGC(display, window, 0, null_mut());

        // data is pointed at the canvas pixels in present
        let image = XCreateImage(
            display,
            XDefaultVisual(display, screen),
            XDefaultDepth(display, screen) as u32,
            ZPixmap,
            0,
            null_mut(),
            width,
            height,
            32,
            0,
        );
        if image.is_null() {
            eprintln!("error: XCreateImage failed");
            exit(1);
        }

        Window {
            display,
            window,
            gc,
            image,
            wm_delete,
            width,
            height,
        }
    }
}

fn key(sym: KeySym) -> Option<Key> {
    let sym = sym as u32;
    let key = match sym {
        XK_Escape => Key::Escape,
        XK_space => Key::Space,
        XK_Return => Key::Enter,
        XK_Up => Key::Up,
        XK_Down => Key::Down,
        XK_Left => Key::Left,
        XK_Right => Key::Right,
        XK_a..=XK_z => Key::Char(b'a' + (sym - XK_a) as u8),
        XK_0..=XK_9 => Key::Char(b'0' + (sym - XK_0) as u8),
        _ => return None,
    };
    Some(key)
}

pub unsafe fn poll_event(window: &mut Window) -> Option<Event> {
    unsafe {
        while XPending(window.display) > 0 {
            let mut event: XEvent = std::mem::zeroed();
            XNextEvent(window.display, &mut event);
            match event.get_type() {
                ClientMessage => {
                    if event.client_message.data.get_long(0) as Atom == window.wm_delete {
                        return Some(Event::Quit);
                    }
                }
                KeyPress => {
                    if let Some(k) = key(XLookupKeysym(&mut event.key, 0)) {
                        return Some(Event::KeyDown(k));
                    }
                }
                ButtonPress if event.button.button == Button1 => {
                    return Some(Event::MouseDown(event.button.x, event.button.y));
                }
                ButtonRelease if event.button.button == Button1 => {
                    return Some(Event::MouseUp(event.button.x, event.button.y));
                }
                MotionNotify => {
                    return Some(Event::MouseMove(event.motion.x, event.motion.y));
                }
                _ => {}
            }
        }
        None
    }
}

pub unsafe fn should_quit(window: &mut Window) -> bool {
    unsafe {
        let mut quit = false;
        while let Some(event) = poll_event(window) {
            quit |= is_quit(event);
        }
        quit
    }
}

pub unsafe fn present(window: &Window, canvas: &Canvas) {
    assert!(canvas.width == window.width as usize && canvas.height == window.height as usize);
    unsafe {
        (*window.image).data = canvas.pixels.as_ptr() as *mut c_char;
        XPutImage(
            window.display,
            window.window,
            window.gc,
            window.image,
            0,
            0,
            0,
            0,
            window.width,
            window.height,
        );
        XFlush(window.display);
    }
}

pub unsafe fn close_window(window: Window) {
    unsafe {
        // the pixels belong to the canvas, detach them so XDestroyImage doesn't free them
        (*window.image).data = null_mut();
        XDestroyImage(window.image);
        XFreeGC(window.display, window.gc);
        XDestroyWindow(window.display, window.window);
        XCloseDisplay(window.display);
    }
}
