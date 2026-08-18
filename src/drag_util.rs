use tao::platform::windows::WindowExtWindows;
use tao::window::Window;

#[cfg(target_os = "windows")]
pub fn drag_window_native(window: &Window) {
    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }
    #[link(name = "user32")]
    extern "system" {
        fn ReleaseCapture() -> i32;
        fn GetCursorPos(lpPoint: *mut POINT) -> i32;
        fn PostMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> i32;
    }

    const WM_NCLBUTTONDOWN: u32 = 0x00A1;
    const HTCAPTION: usize = 2;

    let hwnd = window.hwnd() as isize;
    unsafe {
        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        let lparam = ((pt.x & 0xFFFF) as isize) | (((pt.y & 0xFFFF) as isize) << 16);
        ReleaseCapture();
        PostMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION, lparam);
    }
}

#[cfg(target_os = "windows")]
pub fn apply_dark_window_attributes(window: &Window, bg_color: (u8, u8, u8)) {
    #[link(name = "gdi32")]
    extern "system" {
        fn CreateSolidBrush(color: u32) -> isize;
    }
    #[link(name = "user32")]
    extern "system" {
        fn SetClassLongPtrW(hwnd: isize, nindex: i32, dwnewlong: isize) -> usize;
    }
    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: isize,
            attr: u32,
            value: *const std::ffi::c_void,
            size: u32,
        ) -> i32;
    }

    let hwnd = window.hwnd() as isize;
    // COLORREF is 0x00BBGGRR
    let colorref = (bg_color.0 as u32) | ((bg_color.1 as u32) << 8) | ((bg_color.2 as u32) << 16);
    unsafe {
        let brush = CreateSolidBrush(colorref);
        const GCLP_HBRBACKGROUND: i32 = -10;
        SetClassLongPtrW(hwnd, GCLP_HBRBACKGROUND, brush);

        let dark_mode: i32 = 1;
        let _ = DwmSetWindowAttribute(
            hwnd,
            20,
            &dark_mode as *const _ as *const _,
            std::mem::size_of::<i32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            19,
            &dark_mode as *const _ as *const _,
            std::mem::size_of::<i32>() as u32,
        );
    }
}

