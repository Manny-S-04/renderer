#![allow(clippy::missing_safety_doc)]

// safety: one Window per thread, call everything from the thread that opened it

use std::collections::VecDeque;
use std::ffi::c_void;
use std::process::exit;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::canvas::Canvas;
use crate::event::{Event, Key, is_quit};

// fixed size, the canvas doesn't scale
const STYLE: u32 = WS_OVERLAPPEDWINDOW & !WS_THICKFRAME & !WS_MAXIMIZEBOX;

pub struct Window {
    hwnd: HWND,
    hdc: HDC,
    info: BITMAPINFO,
    width: i32,
    height: i32,
    // heap allocated so window_proc can reach it, freed in close_window
    events: *mut VecDeque<Event>,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn key(vk: WPARAM) -> Option<Key> {
    let vk = vk as u16;
    let key = match vk {
        VK_ESCAPE => Key::Escape,
        VK_SPACE => Key::Space,
        VK_RETURN => Key::Enter,
        VK_UP => Key::Up,
        VK_DOWN => Key::Down,
        VK_LEFT => Key::Left,
        VK_RIGHT => Key::Right,
        0x41..=0x5a => Key::Char(b'a' + (vk - 0x41) as u8),
        0x30..=0x39 => Key::Char(b'0' + (vk - 0x30) as u8),
        _ => return None,
    };
    Some(key)
}

fn mouse_x(lparam: LPARAM) -> i32 {
    (lparam & 0xffff) as i16 as i32
}

fn mouse_y(lparam: LPARAM) -> i32 {
    ((lparam >> 16) & 0xffff) as i16 as i32
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        // null until open_window sets it, messages sent during CreateWindowExW land here
        let events = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut VecDeque<Event>;
        if events.is_null() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        match msg {
            WM_CLOSE => {
                (*events).push_back(Event::Quit);
                0
            }
            WM_KEYDOWN => {
                if let Some(k) = key(wparam) {
                    (*events).push_back(Event::KeyDown(k));
                }
                0
            }
            WM_LBUTTONDOWN => {
                // keep getting mouse messages if a drag leaves the window
                SetCapture(hwnd);
                (*events).push_back(Event::MouseDown(mouse_x(lparam), mouse_y(lparam)));
                0
            }
            WM_LBUTTONUP => {
                ReleaseCapture();
                (*events).push_back(Event::MouseUp(mouse_x(lparam), mouse_y(lparam)));
                0
            }
            WM_MOUSEMOVE => {
                (*events).push_back(Event::MouseMove(mouse_x(lparam), mouse_y(lparam)));
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

pub unsafe fn open_window(title: &str, width: u32, height: u32) -> Window {
    unsafe {
        let instance = GetModuleHandleW(null());
        let class_name = wide("renderer");
        let title = wide(title);

        let mut class: WNDCLASSW = std::mem::zeroed();
        class.style = CS_OWNDC;
        class.lpfnWndProc = Some(window_proc);
        class.hInstance = instance;
        class.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
        class.lpszClassName = class_name.as_ptr();
        if RegisterClassW(&class) == 0 {
            eprintln!("error: RegisterClassW failed");
            exit(1);
        }

        // width and height are the client area, the window adds the border and title bar
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        AdjustWindowRect(&mut rect, STYLE, 0);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            STYLE | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            rect.right - rect.left,
            rect.bottom - rect.top,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            eprintln!("error: CreateWindowExW failed");
            exit(1);
        }

        let events: *mut VecDeque<Event> = Box::into_raw(Box::new(VecDeque::new()));
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, events as isize);

        let hdc = GetDC(hwnd);

        let mut info: BITMAPINFO = std::mem::zeroed();
        info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        info.bmiHeader.biWidth = width as i32;
        // negative height means top-down rows, same as the canvas
        info.bmiHeader.biHeight = -(height as i32);
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        info.bmiHeader.biCompression = BI_RGB;

        Window {
            hwnd,
            hdc,
            info,
            width: width as i32,
            height: height as i32,
            events,
        }
    }
}

pub unsafe fn poll_event(window: &mut Window) -> Option<Event> {
    unsafe {
        if (*window.events).is_empty() {
            let mut msg: MSG = std::mem::zeroed();
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        (*window.events).pop_front()
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
        StretchDIBits(
            window.hdc,
            0,
            0,
            window.width,
            window.height,
            0,
            0,
            window.width,
            window.height,
            canvas.pixels.as_ptr() as *const c_void,
            &window.info,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
    }
}

pub unsafe fn close_window(window: Window) {
    unsafe {
        // window_proc must not touch the queue once it's freed
        SetWindowLongPtrW(window.hwnd, GWLP_USERDATA, 0);
        ReleaseDC(window.hwnd, window.hdc);
        DestroyWindow(window.hwnd);
        drop(Box::from_raw(window.events));
    }
}
