use crate::webview::{self, WebView};
use std::{
    cell::RefCell,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::COLOR_WINDOW,
    System::{Com::*, LibraryLoader::*, Threading::*},
    UI::{HiDpi::*, Shell::*, WindowsAndMessaging::*},
};
thread_local! { static VIEW: RefCell<Option<WebView>> = const { RefCell::new(None) }; }
const CLASS: &str = "CoreScope.Desktop.Window";
const WM_TRAY: u32 = WM_APP + 1;
const RESTART_ACK: usize = 0x4353;
fn elevation_restart_message() -> u32 {
    static MESSAGE: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *MESSAGE.get_or_init(|| unsafe {
        RegisterWindowMessageW(wide("CoreScope.Desktop.ExitForElevation").as_ptr())
    })
}
thread_local! { static TRAY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
unsafe fn tray(hwnd: HWND, action: u32) {
    let mut n: NOTIFYICONDATAW = std::mem::zeroed();
    n.cbSize = std::mem::size_of_val(&n) as u32;
    n.hWnd = hwnd;
    n.uID = 1;
    n.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    n.uCallbackMessage = WM_TRAY;
    n.hIcon = icon();
    let tip = wide("CoreScope · CPU / RAM / °C");
    n.szTip[..tip.len()].copy_from_slice(&tip);
    let ok = Shell_NotifyIconW(action, &n) != 0;
    if action == NIM_ADD {
        TRAY.with(|t| t.set(ok));
    }
}
unsafe fn menu(hwnd: HWND) {
    let en = crate::settings::get().language == "en";
    let m = CreatePopupMenu();
    for (id, ru, eng) in [
        (1, "Открыть CoreScope", "Open CoreScope"),
        (2, "Русский / English", "Русский / English"),
        (3, "Закрывать в трей", "Close to tray"),
        (4, "Включить расширенные датчики", "Enable extended sensors"),
        (5, "Выход", "Exit"),
    ] {
        let flags = MF_STRING
            | if id == 3 && crate::settings::get().close_to_tray {
                MF_CHECKED
            } else {
                0
            };
        AppendMenuW(m, flags, id, wide(if en { eng } else { ru }).as_ptr());
    }
    let mut p: POINT = std::mem::zeroed();
    GetCursorPos(&mut p);
    SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(
        m,
        TPM_RETURNCMD | TPM_RIGHTBUTTON,
        p.x,
        p.y,
        0,
        hwnd,
        null(),
    );
    DestroyMenu(m);
    match cmd {
        1 => {
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
        }
        2 | 3 => {
            let mut s = crate::settings::get();
            if cmd == 2 {
                s.language = if en { "ru" } else { "en" }.into();
            } else {
                s.close_to_tray = !s.close_to_tray;
            }
            if let Err(e) = crate::settings::save(s) {
                error(&e);
            }
        }
        4 => {
            if let Err(e) = crate::settings::start_sensors() {
                error(&e);
            }
        }
        5 => {
            DestroyWindow(hwnd);
        }
        _ => {}
    }
    PostMessageW(hwnd, WM_NULL, 0, 0);
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
unsafe fn icon() -> HICON {
    let bytes = include_bytes!("../assets/corescope.ico");
    static ICON: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *ICON.get_or_init(|| {
        CreateIconFromResourceEx(
            bytes[22..].as_ptr(),
            (bytes.len() - 22) as u32,
            1,
            0x30000,
            32,
            32,
            LR_DEFAULTCOLOR,
        ) as usize
    }) as HICON
}
pub fn error(s: &str) {
    unsafe {
        MessageBoxW(
            null_mut(),
            wide(s).as_ptr(),
            wide("CoreScope").as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}
pub fn activate_existing() -> Result<bool, String> {
    unsafe {
        let h = FindWindowW(wide(CLASS).as_ptr(), null());
        if h.is_null() {
            return Ok(false);
        }
        if crate::platform::process_elevated(std::process::id())? {
            let mut pid = 0;
            GetWindowThreadProcessId(h, &mut pid);
            if !crate::platform::process_elevated(pid)? {
                let process = OpenProcess(
                    PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
                    0,
                    pid,
                );
                if process.is_null() {
                    return Err(std::io::Error::last_os_error().to_string());
                }
                let message = elevation_restart_message();
                let mut reply = 0;
                let sent = message != 0
                    && SendMessageTimeoutW(
                        h,
                        message,
                        std::process::id() as usize,
                        0,
                        SMTO_ABORTIFHUNG | SMTO_BLOCK,
                        2000,
                        &mut reply,
                    ) != 0;
                let result = if !sent || reply != RESTART_ACK {
                    Err(if crate::settings::get().language == "en" {
                        "An older CoreScope is already running without administrator privileges. Exit it using the tray menu, then start CoreScope as administrator again."
                    } else {
                        "Старая копия CoreScope уже работает без прав администратора. Выберите «Выход» в её трее, затем снова запустите CoreScope от администратора."
                    }.into())
                } else if WaitForSingleObject(process, 10_000) == WAIT_OBJECT_0 {
                    Ok(false)
                } else {
                    Err(if crate::settings::get().language == "en" {
                        "The previous CoreScope has not finished shutting down. Wait for it to exit, then start CoreScope as administrator again."
                    } else {
                        "Предыдущая копия CoreScope ещё завершает работу. Дождитесь её выхода и повторите запуск от администратора."
                    }.into())
                };
                CloseHandle(process);
                return result;
            }
        }
        if IsIconic(h) != 0 {
            ShowWindow(h, SW_RESTORE);
        } else {
            ShowWindow(h, SW_SHOW);
        }
        SetForegroundWindow(h);
        Ok(true)
    }
}
fn external(url: &str) {
    if url.starts_with("https://") {
        unsafe {
            ShellExecuteW(
                null_mut(),
                wide("open").as_ptr(),
                wide(url).as_ptr(),
                null(),
                null(),
                SW_SHOWNORMAL,
            );
        }
    }
}
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let restart = elevation_restart_message();
    if restart != 0 && msg == restart {
        if crate::platform::process_elevated(std::process::id()) == Ok(false)
            && u32::try_from(w)
                .ok()
                .is_some_and(|pid| crate::platform::process_elevated(pid) == Ok(true))
        {
            VIEW.with(|v| {
                v.borrow_mut().take();
            });
            DestroyWindow(hwnd);
            return RESTART_ACK as isize;
        }
        return 0;
    }
    if msg == RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()) {
        tray(hwnd, NIM_ADD);
        return 0;
    }
    match msg {
        WM_TRAY => {
            match l as u32 {
                WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
                    ShowWindow(hwnd, SW_RESTORE);
                    SetForegroundWindow(hwnd);
                }
                WM_RBUTTONUP | WM_CONTEXTMENU => menu(hwnd),
                _ => {}
            }
            0
        }
        WM_SIZE => {
            VIEW.with(|v| {
                if let Some(v) = v.borrow().as_ref() {
                    v.resize((l & 0xffff) as i32, ((l >> 16) & 0xffff) as i32)
                }
            });
            0
        }
        WM_MOVE => {
            VIEW.with(|v| {
                if let Some(v) = v.borrow().as_ref() {
                    v.parent_moved()
                }
            });
            0
        }
        WM_SETFOCUS => {
            VIEW.with(|v| {
                if let Some(v) = v.borrow().as_ref() {
                    v.focus()
                }
            });
            0
        }
        WM_GETMINMAXINFO => {
            let info = l as *mut MINMAXINFO;
            if !info.is_null() {
                (*info).ptMinTrackSize.x = 800;
                (*info).ptMinTrackSize.y = 600;
            }
            0
        }
        WM_DPICHANGED => {
            let r = l as *const RECT;
            if !r.is_null() {
                let r = &*r;
                SetWindowPos(
                    hwnd,
                    null_mut(),
                    r.left,
                    r.top,
                    r.right - r.left,
                    r.bottom - r.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            0
        }
        WM_CLOSE => {
            if crate::settings::get().close_to_tray && TRAY.with(|t| t.get()) {
                ShowWindow(hwnd, SW_HIDE);
                return 0;
            }
            VIEW.with(|v| {
                v.borrow_mut().take();
            });
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            tray(hwnd, NIM_DELETE);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}
pub fn run(url: String) -> Result<(), String> {
    unsafe {
        let mutex = windows_sys::Win32::System::Threading::CreateMutexW(
            null(),
            0,
            wide("CoreScope.Desktop").as_ptr(),
        );
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let hr = CoInitializeEx(null(), COINIT_APARTMENTTHREADED as u32);
        if hr < 0 {
            return Err(format!("COM: 0x{hr:08x}"));
        }
        let result = run_initialized(url);
        VIEW.with(|v| {
            v.borrow_mut().take();
        });
        CoUninitialize();
        if !mutex.is_null() {
            CloseHandle(mutex);
        }
        result
    }
}
unsafe fn run_initialized(url: String) -> Result<(), String> {
    webview::runtime_version()?;
    let module = GetModuleHandleW(null());
    let class = wide(CLASS);
    let mut wc: WNDCLASSW = std::mem::zeroed();
    wc.lpfnWndProc = Some(wndproc);
    wc.hInstance = module;
    wc.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
    wc.hIcon = icon();
    wc.hbrBackground = (COLOR_WINDOW + 1) as usize as _;
    wc.lpszClassName = class.as_ptr();
    if RegisterClassW(&wc) == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let scale = GetDpiForSystem().max(96) as f32 / 96.;
    let sw = GetSystemMetrics(SM_CXSCREEN);
    let sh = GetSystemMetrics(SM_CYSCREEN);
    let width = (sw * 9 / 10).min((1380. * scale) as i32);
    let height = (sh * 9 / 10).min((920. * scale) as i32);
    let hwnd = CreateWindowExW(
        0,
        class.as_ptr(),
        wide("CoreScope").as_ptr(),
        WS_OVERLAPPEDWINDOW,
        (sw - width) / 2,
        (sh - height) / 2,
        width,
        height,
        null_mut(),
        null_mut(),
        module,
        null(),
    );
    if hwnd.is_null() {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let profile = crate::settings::directory().join("WebView2");
    let created = webview::create(
        hwnd,
        &profile,
        url.clone(),
        url,
        external,
        Box::new(move |r| match r {
            Ok(view) => {
                let mut rect: RECT = std::mem::zeroed();
                GetClientRect(hwnd, &mut rect);
                view.resize(rect.right, rect.bottom);
                view.set_visible(true);
                VIEW.with(|v| *v.borrow_mut() = Some(view));
            }
            Err(e) => {
                error(&e);
                DestroyWindow(hwnd);
            }
        }),
    );
    if let Err(e) = created {
        DestroyWindow(hwnd);
        return Err(e);
    }
    tray(hwnd, NIM_ADD);
    if !std::env::args().any(|a| a == "--tray") || !TRAY.with(|t| t.get()) {
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
    }
    let mut msg: MSG = std::mem::zeroed();
    loop {
        let code = GetMessageW(&mut msg, null_mut(), 0, 0);
        if code == 0 {
            break;
        }
        if code < 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    Ok(())
}
