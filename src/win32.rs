#![allow(non_camel_case_types, non_snake_case, dead_code)]

use std::ffi::c_void;

pub type HWND = isize;
pub type HINSTANCE = isize;
pub type HDC = isize;
pub type HFONT = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
pub type UINT = u32;
pub type DWORD = u32;
pub type WORD = u16;
pub type BOOL = i32;
pub type LPCWSTR = *const u16;
pub type WNDPROC = extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;

pub const WM_DESTROY: UINT = 0x0002;
pub const WM_PAINT: UINT = 0x000F;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_LBUTTONDOWN: UINT = 0x0201;
pub const WM_LBUTTONUP: UINT = 0x0202;
pub const WM_MOUSEMOVE: UINT = 0x0200;
pub const WM_MOUSEWHEEL: UINT = 0x020A;
pub const WM_KEYDOWN: UINT = 0x0100;
pub const WM_COMMAND: UINT = 0x0111;
pub const WM_CTLCOLOREDIT: UINT = 0x0133;
pub const WM_ERASEBKGND: UINT = 0x0014;

pub const VK_CONTROL: i32 = 0x11;

pub const WS_OVERLAPPEDWINDOW: DWORD = 0x00CF0000;
pub const WS_VISIBLE: DWORD = 0x10000000;
pub const WS_CHILD: DWORD = 0x40000000;
pub const WS_BORDER: DWORD = 0x00800000;
pub const WS_POPUP: DWORD = 0x80000000;
pub const WS_CAPTION: DWORD = 0x00C00000;
pub const WS_SYSMENU: DWORD = 0x00080000;
pub const CS_HREDRAW: UINT = 0x0002;
pub const CS_VREDRAW: UINT = 0x0001;
pub const SW_SHOW: i32 = 5;

pub const BI_RGB: u32 = 0;
pub const DIB_RGB_COLORS: u32 = 0;
pub const SRCCOPY: u32 = 0x00CC0020;
pub const TRANSPARENT: i32 = 1;

pub const BS_DEFPUSHBUTTON: DWORD = 0x00000001;
pub const EM_REPLACESEL: UINT = 0x00C2;
pub const VK_RETURN: i32 = 0x0D;
pub const GWLP_WNDPROC: i32 = -4;

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: UINT,
    pub style: UINT,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: isize,
    pub hCursor: isize,
    pub hbrBackground: isize,
    pub lpszMenuName: LPCWSTR,
    pub lpszClassName: LPCWSTR,
    pub hIconSm: isize,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: DWORD,
    pub pt_x: i32,
    pub pt_y: i32,
}

#[repr(C)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: BOOL,
    pub rcPaint: RECT,
    pub fRestore: BOOL,
    pub fIncUpdate: BOOL,
    pub rgbReserved: [u8; 32],
}

#[repr(C)]
pub struct BITMAPINFOHEADER {
    pub biSize: u32,
    pub biWidth: i32,
    pub biHeight: i32,
    pub biPlanes: u16,
    pub biBitCount: u16,
    pub biCompression: u32,
    pub biSizeImage: u32,
    pub biXPelsPerMeter: i32,
    pub biYPelsPerMeter: i32,
    pub biClrUsed: u32,
    pub biClrImportant: u32,
}

#[repr(C)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [u32; 1],
}

#[repr(C)]
pub struct SIZE {
    pub cx: i32,
    pub cy: i32,
}

#[repr(C)]
pub struct OPENFILENAMEW {
    pub lStructSize: DWORD,
    pub hwndOwner: HWND,
    pub hInstance: HINSTANCE,
    pub lpstrFilter: LPCWSTR,
    pub lpstrCustomFilter: *mut u16,
    pub nMaxCustFilter: DWORD,
    pub nFilterIndex: DWORD,
    pub lpstrFile: *mut u16,
    pub nMaxFile: DWORD,
    pub lpstrFileTitle: *mut u16,
    pub nMaxFileTitle: DWORD,
    pub lpstrInitialDir: LPCWSTR,
    pub lpstrTitle: LPCWSTR,
    pub Flags: DWORD,
    pub nFileOffset: WORD,
    pub nFileExtension: WORD,
    pub lpstrDefExt: LPCWSTR,
    pub lCustData: LPARAM,
    pub lpfnHook: isize,
    pub lpTemplateName: LPCWSTR,
    pub pvReserved: *mut c_void,
    pub dwReserved: DWORD,
    pub FlagsEx: DWORD,
}

#[link(name = "user32")]
unsafe extern "system" {
    pub fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: LPCWSTR,
        lpWindowName: LPCWSTR,
        dwStyle: DWORD,
        x: i32,
        y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: isize,
        hInstance: HINSTANCE,
        lpParam: *const c_void,
    ) -> HWND;
    pub fn DefWindowProcW(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
    pub fn UpdateWindow(hWnd: HWND) -> BOOL;
    pub fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
    pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    pub fn PostQuitMessage(nExitCode: i32);
    pub fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
    pub fn TextOutW(hdc: HDC, x: i32, y: i32, lpString: LPCWSTR, c: i32) -> BOOL;
    pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    pub fn SetTextColor(hdc: HDC, color: u32) -> u32;
    pub fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub fn GetWindowTextW(hWnd: HWND, lpString: *mut u16, nMaxCount: i32) -> i32;
    pub fn GetKeyState(nVirtKey: i32) -> i16;
    pub fn FillRect(hdc: HDC, lprc: *const RECT, hbr: isize) -> i32;
    pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn SetFocus(hWnd: HWND) -> HWND;
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
    pub fn CallWindowProcW(lpPrevWndFunc: isize, hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn AdjustWindowRectEx(lpRect: *mut RECT, dwStyle: DWORD, bMenu: BOOL, dwExStyle: DWORD) -> BOOL;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    pub fn StretchDIBits(
        hdc: HDC,
        xDest: i32,
        yDest: i32,
        destWidth: i32,
        destHeight: i32,
        xSrc: i32,
        ySrc: i32,
        srcWidth: i32,
        srcHeight: i32,
        lpBits: *const c_void,
        lpbmi: *const BITMAPINFO,
        iUsage: u32,
        rop: u32,
    ) -> i32;
    pub fn CreateFontW(
        cHeight: i32,
        cWidth: i32,
        cEscapement: i32,
        cOrientation: i32,
        cWeight: i32,
        bItalic: DWORD,
        bUnderline: DWORD,
        bStrikeOut: DWORD,
        iCharSet: DWORD,
        iOutPrecision: DWORD,
        iClipPrecision: DWORD,
        iQuality: DWORD,
        iPitchAndFamily: DWORD,
        pszFaceName: LPCWSTR,
    ) -> HFONT;
    pub fn SelectObject(hdc: HDC, hgdiobj: isize) -> isize;
    pub fn DeleteObject(hobj: isize) -> BOOL;
    pub fn GetTextExtentPoint32W(hdc: HDC, lpString: LPCWSTR, c: i32, lpSize: *mut SIZE) -> BOOL;
    pub fn CreateSolidBrush(color: u32) -> isize;
    pub fn SetBkColor(hdc: HDC, color: u32) -> u32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HINSTANCE;
}

#[link(name = "comdlg32")]
unsafe extern "system" {
    pub fn GetOpenFileNameW(a: *mut OPENFILENAMEW) -> BOOL;
    pub fn GetSaveFileNameW(a: *mut OPENFILENAMEW) -> BOOL;
}

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}