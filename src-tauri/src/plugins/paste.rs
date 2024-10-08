use crate::core::app::get_foreground_apps;
use tauri::{
    command, generate_handler,
    plugin::{Builder, TauriPlugin},
    Wry,
};

// 让上一个窗口聚焦（macos）
#[cfg(target_os = "macos")]
fn focus_previous_window() {
    use cocoa::{
        appkit::{NSApplicationActivationOptions, NSRunningApplication},
        base::nil,
    };

    let foreground_apps = get_foreground_apps();

    let process_id = foreground_apps[0].process_id;

    unsafe {
        let app = NSRunningApplication::runningApplicationWithProcessIdentifier(nil, process_id);

        app.activateWithOptions_(
            NSApplicationActivationOptions::NSApplicationActivateIgnoringOtherApps,
        );
    }
}

// 让上一个窗口聚焦（windows）
#[cfg(target_os = "windows")]
fn focus_previous_window() {
    use winapi::{shared::windef::HWND, um::winuser::SetForegroundWindow};
    unsafe {
        let foreground_apps = get_foreground_apps();

        let hwnd = foreground_apps[0] as HWND;

        if hwnd.is_null() {
            println!("Could not get active window");
            return;
        }

        if SetForegroundWindow(hwnd) == 0 {
            println!("Could not focus on window");
        }
    }
}

// 让上一个窗口聚焦（linux）
#[cfg(target_os = "linux")]
fn focus_previous_window() {
    use x11::xlib::{self, XCloseDisplay, XOpenDisplay, XRaiseWindow, XSetInputFocus};

    unsafe {
        let display = XOpenDisplay(std::ptr::null_mut());
        if display.is_null() {
            log::error!("Could not open display");
            return;
        }
        let window = match get_foreground_apps() {
            Some(window) => window,
            None => {
                log::error!("Could not get active window");
                return;
            }
        };

        XRaiseWindow(display, window);
        XSetInputFocus(display, window, xlib::RevertToNone, xlib::CurrentTime);
        XCloseDisplay(display);
    }
}

// 粘贴剪贴板内容（macos）
#[cfg(target_os = "macos")]
#[command]
async fn paste() {
    focus_previous_window();

    let script =
        r#"osascript -e 'tell application "System Events" to keystroke "v" using command down'"#;

    std::process::Command::new("sh")
        .arg("-c")
        .arg(script)
        .output()
        .expect("failed to execute process");
}

// 粘贴剪贴板内容（windows）
#[cfg(target_os = "windows")]
#[command]
async fn paste() {
    use enigo::{
        Direction::{Click, Press, Release},
        Enigo, Key, Keyboard, Settings,
    };

    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    focus_previous_window();

    wait(100);

    enigo.key(Key::Shift, Press).unwrap();
    // insert 的微软虚拟键码：https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
    enigo.key(Key::Other(0x2D), Click).unwrap();
    enigo.key(Key::Shift, Release).unwrap();
}

// 粘贴剪贴板内容（linux）
#[cfg(target_os = "linux")]
#[command]
async fn paste() {
    use rdev::{simulate, EventType, Key};

    fn dispatch(event_type: &EventType) {
        wait(20);

        simulate(event_type).unwrap();
    }

    focus_previous_window();

    wait(100);

    dispatch(&EventType::KeyPress(Key::ShiftLeft));
    dispatch(&EventType::KeyPress(Key::Insert));
    dispatch(&EventType::KeyRelease(Key::Insert));
    dispatch(&EventType::KeyRelease(Key::ShiftLeft));
}

#[cfg(not(target_os = "macos"))]
fn wait(millis: u64) {
    use std::{thread, time};

    thread::sleep(time::Duration::from_millis(millis));
}

pub fn init() -> TauriPlugin<Wry> {
    Builder::new("paste")
        .invoke_handler(generate_handler![paste])
        .build()
}
