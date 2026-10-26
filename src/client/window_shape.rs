//! The window resizes only in the picture's 16:9 where the desktop can be asked for it:
//! on Windows, by answering WM_SIZING. Wayland has no way to ask (xdg-shell knows only a
//! smallest and a largest size), and whatever gets around it (a maximized window, a
//! tiling desktop) still shows the picture in 16:9 between black bars (canvas.rs).
//!
//! miniquad gives no handle to its window, so the window is found again among this
//! process's.

use crate::client::canvas::{VIEW_HEIGHT, VIEW_WIDTH};

/// Frames to keep looking for the window: it is listed by the desktop a moment after
/// the game's first frame.
const SEARCH_FRAMES: u32 = 128;

#[derive(Default)]
pub struct WindowShape {
    settled: bool,
    frames_searched: u32,
}

impl WindowShape {
    /// Called every frame; does its work once, as soon as the window can be found.
    pub fn keep_16_9(&mut self) {
        if self.settled {
            return;
        }
        self.frames_searched += 1;
        self.settled = platform::keep_aspect(VIEW_WIDTH as i32, VIEW_HEIGHT as i32) || self.frames_searched >= SEARCH_FRAMES;
    }
}

#[cfg(windows)]
mod platform {
    use std::sync::atomic::{AtomicI32, AtomicIsize, Ordering};

    type Hwnd = isize;

    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn EnumWindows(callback: extern "system" fn(Hwnd, isize) -> i32, data: isize) -> i32;
        fn GetWindowThreadProcessId(window: Hwnd, process: *mut u32) -> u32;
        fn IsWindowVisible(window: Hwnd) -> i32;
        fn SetWindowLongPtrW(window: Hwnd, index: i32, value: isize) -> isize;
        fn CallWindowProcW(previous: isize, window: Hwnd, message: u32, wparam: usize, lparam: isize) -> isize;
        fn GetWindowRect(window: Hwnd, rect: *mut Rect) -> i32;
        fn GetClientRect(window: Hwnd, rect: *mut Rect) -> i32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcessId() -> u32;
    }

    const GWLP_WNDPROC: i32 = -4;
    const WM_SIZING: u32 = 0x0214;
    const WMSZ_TOP: usize = 3;
    const WMSZ_TOPLEFT: usize = 4;
    const WMSZ_TOPRIGHT: usize = 5;
    const WMSZ_BOTTOM: usize = 6;

    /// miniquad's window procedure, which everything but WM_SIZING still goes to.
    static MINIQUAD_PROC: AtomicIsize = AtomicIsize::new(0);
    static FOUND: AtomicIsize = AtomicIsize::new(0);
    static ASPECT: (AtomicI32, AtomicI32) = (AtomicI32::new(16), AtomicI32::new(9));

    /// Returns whether it is done: the window found and answering WM_SIZING.
    pub fn keep_aspect(width: i32, height: i32) -> bool {
        ASPECT.0.store(width, Ordering::Relaxed);
        ASPECT.1.store(height, Ordering::Relaxed);
        // Safe: the callback only reads its arguments and stores a window handle.
        let window = unsafe {
            FOUND.store(0, Ordering::Relaxed);
            EnumWindows(own_visible_window, 0);
            FOUND.load(Ordering::Relaxed)
        };
        if window == 0 {
            return false;
        }
        // Safe: the window is this process's, and the new procedure passes everything
        // on to the one it replaces.
        unsafe {
            MINIQUAD_PROC.store(SetWindowLongPtrW(window, GWLP_WNDPROC, keep_16_9 as *const () as isize), Ordering::Relaxed);
        }
        true
    }

    extern "system" fn own_visible_window(window: Hwnd, _: isize) -> i32 {
        let mut process = 0;
        // Safe: plain queries about a window handle the system passed in.
        unsafe {
            GetWindowThreadProcessId(window, &mut process);
            if process == GetCurrentProcessId() && IsWindowVisible(window) != 0 {
                FOUND.store(window, Ordering::Relaxed);
                return 0;
            }
        }
        1
    }

    extern "system" fn keep_16_9(window: Hwnd, message: u32, wparam: usize, lparam: isize) -> isize {
        // Safe: for WM_SIZING lparam points at the window rectangle being dragged.
        unsafe {
            if message == WM_SIZING {
                fit(window, wparam, &mut *(lparam as *mut Rect));
            }
            CallWindowProcW(MINIQUAD_PROC.load(Ordering::Relaxed), window, message, wparam, lparam)
        }
    }

    /// The dragged rectangle made 16:9 inside its frame: a side edge sets the height
    /// from the width, the top or bottom edge the width from the height, a corner moves
    /// the edge beside it.
    fn fit(window: Hwnd, edge: usize, rect: &mut Rect) {
        let (mut outer, mut inner) = (Rect { left: 0, top: 0, right: 0, bottom: 0 }, Rect { left: 0, top: 0, right: 0, bottom: 0 });
        // Safe: the window is the one WM_SIZING came for, and both rectangles are ours.
        unsafe {
            GetWindowRect(window, &mut outer);
            GetClientRect(window, &mut inner);
        }
        let frame_width = (outer.right - outer.left) - (inner.right - inner.left);
        let frame_height = (outer.bottom - outer.top) - (inner.bottom - inner.top);
        let (aspect_x, aspect_y) = (ASPECT.0.load(Ordering::Relaxed), ASPECT.1.load(Ordering::Relaxed));
        let width = rect.right - rect.left - frame_width;
        let height = rect.bottom - rect.top - frame_height;
        match edge {
            WMSZ_TOP | WMSZ_BOTTOM => rect.right = rect.left + height * aspect_x / aspect_y + frame_width,
            WMSZ_TOPLEFT | WMSZ_TOPRIGHT => rect.top = rect.bottom - width * aspect_y / aspect_x - frame_height,
            // The left or right edge, and the bottom corners.
            _ => rect.bottom = rect.top + width * aspect_y / aspect_x + frame_height,
        }
    }
}

#[cfg(not(windows))]
mod platform {
    /// Elsewhere the picture keeps its shape between black bars only.
    pub fn keep_aspect(_: i32, _: i32) -> bool {
        true
    }
}
