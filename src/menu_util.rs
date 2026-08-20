use tao::platform::windows::WindowExtWindows;
use tao::window::Window;

#[cfg(target_os = "windows")]
pub fn show_native_bookmark_context_menu(window: &Window) -> Option<u32> {
    #[repr(C)]
    #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
    struct POINT {
        x: i32,
        y: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn CreatePopupMenu() -> isize;
        fn AppendMenuW(
            hmenu: isize,
            flags: u32,
            id_new_item: usize,
            lp_new_item: *const u16,
        ) -> i32;
        fn TrackPopupMenu(
            hmenu: isize,
            flags: u32,
            x: i32,
            y: i32,
            reserved: i32,
            hwnd: isize,
            prc_rect: *const std::ffi::c_void,
        ) -> i32;
        fn DestroyMenu(hmenu: isize) -> i32;
        fn GetCursorPos(lp_point: *mut POINT) -> i32;
        fn SetForegroundWindow(hwnd: isize) -> i32;
    }

    const MF_STRING: u32 = 0x0000;
    const MF_SEPARATOR: u32 = 0x0800;
    const TPM_RETURNCMD: u32 = 0x0100;
    const TPM_NONOTIFY: u32 = 0x0080;
    const TPM_RIGHTBUTTON: u32 = 0x0002;

    let hwnd = window.hwnd();
    let mut pt = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut pt);
        let hmenu = CreatePopupMenu();
        if hmenu == 0 {
            return None;
        }

        let str_open: Vec<u16> = "Open in New Tab\0".encode_utf16().collect();
        let str_copy: Vec<u16> = "Copy Link Address\0".encode_utf16().collect();
        let str_delete: Vec<u16> = "Delete Bookmark\0".encode_utf16().collect();

        AppendMenuW(hmenu, MF_STRING, 1, str_open.as_ptr());
        AppendMenuW(hmenu, MF_STRING, 2, str_copy.as_ptr());
        AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(hmenu, MF_STRING, 3, str_delete.as_ptr());

        SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(
            hmenu,
            TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
            pt.x,
            pt.y,
            0,
            hwnd,
            std::ptr::null(),
        );
        DestroyMenu(hmenu);

        if cmd > 0 {
            Some(cmd as u32)
        } else {
            None
        }
    }
}

#[cfg(target_os = "windows")]
pub fn copy_to_clipboard(text: &str) {
    #[link(name = "user32")]
    extern "system" {
        fn OpenClipboard(hwnd: isize) -> i32;
        fn CloseClipboard() -> i32;
        fn EmptyClipboard() -> i32;
        fn SetClipboardData(format: u32, hmem: isize) -> isize;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalAlloc(flags: u32, bytes: usize) -> isize;
        fn GlobalLock(hmem: isize) -> *mut u8;
        fn GlobalUnlock(hmem: isize) -> i32;
    }

    const CF_UNICODETEXT: u32 = 13;
    const GMEM_MOVEABLE: u32 = 0x0002;

    let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let byte_len = utf16.len() * 2;

    unsafe {
        if OpenClipboard(0) != 0 {
            EmptyClipboard();
            let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_len);
            if hmem != 0 {
                let ptr = GlobalLock(hmem);
                if !ptr.is_null() {
                    std::ptr::copy_nonoverlapping(utf16.as_ptr() as *const u8, ptr, byte_len);
                    GlobalUnlock(hmem);
                    SetClipboardData(CF_UNICODETEXT, hmem);
                }
            }
            CloseClipboard();
        }
    }
}
