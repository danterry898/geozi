use crate::win32::*;

const IDC_OK: isize = 102;
const DLG_W: i32 = 240;
const DLG_H: i32 = 130;

const FN_DLG_W: i32 = 300;
const HELPER_ID_BASE: isize = 200;
const HELPER_COLS: i32 = 4;
const HELPER_BTN_W: i32 = 60;
const HELPER_BTN_H: i32 = 26;
const HELPER_GRID_TOP: i32 = 54;

const HELPER_BUTTONS: &[(&str, &str)] = &[
    ("+", "+"), ("\u{2212}", "-"), ("\u{00D7}", "*"), ("\u{00F7}", "/"),
    ("x", "x"), ("x\u{00B2}", "^2"), ("^", "^"), ("\u{221A}(", "sqrt("),
    ("exp(", "exp("), ("ln(", "ln("), ("sin(", "sin("), ("cos(", "cos("),
    ("tan(", "tan("), ("(", "("), (")", ")"), ("\u{03C0}", "pi"),
];

const HELPER_ROWS: i32 = (HELPER_BUTTONS.len() as i32 + HELPER_COLS - 1) / HELPER_COLS;
const HELPER_GRID_HEIGHT: i32 = HELPER_ROWS * (HELPER_BTN_H + 4);
const FN_DLG_H: i32 = HELPER_GRID_TOP + HELPER_GRID_HEIGHT + 6 + 28 + 20;

const RESULT_TEXT_LEN: usize = 128;

static mut RESULT_TEXT: [u16; RESULT_TEXT_LEN] = [0; RESULT_TEXT_LEN];
static mut DIALOG_OPEN: bool = false;
static mut DIALOG_HWND: HWND = 0;
static mut EDIT_HWND: HWND = 0;
static mut EDIT_ORIG_PROC: isize = 0;
static mut BG_COLOR: u32 = 0x00303030;
static mut TEXT_COLOR: u32 = 0x00ffffff;
static mut EDIT_BRUSH: isize = 0;

extern "system" fn dialog_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                let mut rect: RECT = std::mem::zeroed();
                GetClientRect(hwnd, &mut rect);
                let brush = CreateSolidBrush(BG_COLOR);
                FillRect(hdc, &rect, brush);
                DeleteObject(brush);
                EndPaint(hwnd, &ps);
                0
            }
            WM_CTLCOLOREDIT => {
                let hdc_edit: HDC = wparam as HDC;
                SetTextColor(hdc_edit, TEXT_COLOR);
                SetBkColor(hdc_edit, BG_COLOR);
                EDIT_BRUSH
            }
            WM_COMMAND => {
                let id = (wparam & 0xffff) as isize;
                if id == IDC_OK {
                    GetWindowTextW(EDIT_HWND, (&raw mut RESULT_TEXT) as *mut u16, RESULT_TEXT_LEN as i32);
                    DIALOG_OPEN = false;
                    DestroyWindow(hwnd);
                } else if id >= HELPER_ID_BASE {
                    let idx = (id - HELPER_ID_BASE) as usize;
                    if let Some(&(_, token)) = HELPER_BUTTONS.get(idx) {
                        let wide = to_wide(token);
                        SendMessageW(EDIT_HWND, EM_REPLACESEL, 1, wide.as_ptr() as LPARAM);
                    }
                }
                0
            }
            WM_DESTROY => {
                DIALOG_OPEN = false;
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

extern "system" fn edit_subclass_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if msg == WM_KEYDOWN && wparam == VK_RETURN as usize {
            SendMessageW(DIALOG_HWND, WM_COMMAND, IDC_OK as usize, 0);
            return 0;
        }
        CallWindowProcW(EDIT_ORIG_PROC, hwnd, msg, wparam, lparam)
    }
}

pub fn register_class(hinstance: HINSTANCE) {
    unsafe {
        let class_name = to_wide("GeoziValueDialog");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: dialog_proc,
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
    }
}

/// Creates the popup window shared by ask_value and ask_function, and resets the dialog's global state.
/// Returns 0 if creation fails (the calling code must then clean up EDIT_BRUSH).
fn open_dialog(owner: HWND, title: &str, width: i32, height: i32, bg: u32, text_color: u32) -> HWND {
    unsafe {
        DIALOG_OPEN = true;
        RESULT_TEXT = [0; RESULT_TEXT_LEN];
        BG_COLOR = bg;
        TEXT_COLOR = text_color;
        EDIT_BRUSH = CreateSolidBrush(bg);

        let hinstance = GetModuleHandleW(std::ptr::null());
        let class_name = to_wide("GeoziValueDialog");
        let wide_title = to_wide(title);
        let style = WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE;

        let mut rect = RECT { left: 0, top: 0, right: width, bottom: height };
        AdjustWindowRectEx(&mut rect, style, 0, 0);
        let outer_w = rect.right - rect.left;
        let outer_h = rect.bottom - rect.top;

        let mut owner_rect: RECT = std::mem::zeroed();
        GetWindowRect(owner, &mut owner_rect);
        let cx = (owner_rect.left + owner_rect.right) / 2;
        let cy = (owner_rect.top + owner_rect.bottom) / 2;

        let dlg_hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            wide_title.as_ptr(),
            style,
            cx - outer_w / 2, cy - outer_h / 2, outer_w, outer_h,
            owner, 0, hinstance, std::ptr::null(),
        );
        if dlg_hwnd == 0 {
            DIALOG_OPEN = false;
        } else {
            DIALOG_HWND = dlg_hwnd;
            EnableWindow(owner, 0);
        }
        dlg_hwnd
    }
}


fn run_message_loop() {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while DIALOG_OPEN {
            if GetMessageW(&mut msg, 0, 0, 0) == 0 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// Reactivates the owner window and gives it keyboard focus again. Without this SetFocus, Windows
/// won't restore it automatically after a non-modal popup is destroyed and the main window
/// stops receiving keyboard input (including Ctrl+Z) until you click on it again.
fn close_dialog(owner: HWND) {
    unsafe {
        EnableWindow(owner, 1);
        SetFocus(owner);
        DeleteObject(EDIT_BRUSH);
    }
}


#[allow(clippy::deref_addrof)]
fn take_result_text() -> String {
    unsafe {
        let text: &[u16; RESULT_TEXT_LEN] = &*(&raw const RESULT_TEXT);
        let len = text.iter().position(|&c| c == 0).unwrap_or(0);
        String::from_utf16_lossy(&text[..len])
    }
}

pub fn ask_value(owner: HWND, title: &str, bg: u32, text_color: u32) -> Option<f64> {
    let dlg_hwnd = open_dialog(owner, title, DLG_W, DLG_H, bg, text_color);
    if dlg_hwnd == 0 {
        unsafe { DeleteObject(EDIT_BRUSH) };
        return None;
    }

    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());
        let edit_class = to_wide("EDIT");
        EDIT_HWND = CreateWindowExW(
            0,
            edit_class.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_BORDER,
            20, 20, 190, 24,
            dlg_hwnd, 0, hinstance, std::ptr::null(),
        );

        let button_class = to_wide("BUTTON");
        let ok_label = to_wide("OK");
        CreateWindowExW(
            0,
            button_class.as_ptr(),
            ok_label.as_ptr(),
            WS_CHILD | WS_VISIBLE,
            75, 60, 80, 28,
            dlg_hwnd, IDC_OK, hinstance, std::ptr::null(),
        );

        SetFocus(EDIT_HWND);
    }

    run_message_loop();
    close_dialog(owner);
    take_result_text().trim().parse::<f64>().ok()
}


pub fn ask_function(owner: HWND, bg: u32, text_color: u32) -> Option<String> {
    let dlg_hwnd = open_dialog(owner, "f(x) =", FN_DLG_W, FN_DLG_H, bg, text_color);
    if dlg_hwnd == 0 {
        unsafe { DeleteObject(EDIT_BRUSH) };
        return None;
    }

    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());
        let edit_class = to_wide("EDIT");
        EDIT_HWND = CreateWindowExW(
            0,
            edit_class.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_BORDER,
            20, 20, FN_DLG_W - 40, 24,
            dlg_hwnd, 0, hinstance, std::ptr::null(),
        );
        EDIT_ORIG_PROC = SetWindowLongPtrW(EDIT_HWND, GWLP_WNDPROC, edit_subclass_proc as usize as isize);

        let button_class = to_wide("BUTTON");
        for (i, &(label, _)) in HELPER_BUTTONS.iter().enumerate() {
            let col = i as i32 % HELPER_COLS;
            let row = i as i32 / HELPER_COLS;
            let wide_label = to_wide(label);
            CreateWindowExW(
                0,
                button_class.as_ptr(),
                wide_label.as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20 + col * (HELPER_BTN_W + 4),
                HELPER_GRID_TOP + row * (HELPER_BTN_H + 4),
                HELPER_BTN_W, HELPER_BTN_H,
                dlg_hwnd, HELPER_ID_BASE + i as isize, hinstance, std::ptr::null(),
            );
        }

        let ok_y = HELPER_GRID_TOP + HELPER_GRID_HEIGHT + 6;
        let ok_label = to_wide("Plot");
        CreateWindowExW(
            0,
            button_class.as_ptr(),
            ok_label.as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
            (FN_DLG_W - 80) / 2, ok_y, 80, 28,
            dlg_hwnd, IDC_OK, hinstance, std::ptr::null(),
        );

        SetFocus(EDIT_HWND);
    }

    run_message_loop();
    close_dialog(owner);
    let text = take_result_text();
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}