mod app;
mod control;
mod dialog;
mod expr;
mod geometry;
mod render;
mod theme;
mod ui;
mod storage;
mod win32;

use win32::*;

fn main() {
    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());
        let class_name = to_wide("Geozi");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: app::window_proc,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: 0,
            hCursor: 0,
            hbrBackground: 0,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: 0,
        };
        RegisterClassExW(&wc);
        dialog::register_class(hinstance);

        let title = to_wide("Geozi");
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            100, 100, 900, 700,
            0, 0, hinstance, std::ptr::null(),
        );

        let mut client_rect: RECT = std::mem::zeroed();
        GetClientRect(hwnd, &mut client_rect);
        app::init(client_rect.right - client_rect.left, client_rect.bottom - client_rect.top);

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}