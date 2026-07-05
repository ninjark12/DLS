use std::cell::Cell;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use serde::{Deserialize, Serialize};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    System::Threading::{
        GetCurrentThreadId, OpenProcess, QueryFullProcessImageNameW,
        PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, GetWindowThreadProcessId,
            PostThreadMessageW, TranslateMessage,
            EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT, WM_APP,
            WM_QUIT,
        },
    },
};

use crate::config::Config;
use crate::hid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitcherEvent {
    pub current_exe: String,
    pub current_layer: u8,
}

thread_local! {
    static THREAD_ID: Cell<u32> = Cell::new(0);
}

unsafe extern "system" fn hook_callback(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    let thread_id = THREAD_ID.with(|tid| tid.get());
    let _ = PostThreadMessageW(thread_id, WM_APP, WPARAM(hwnd.0 as usize), LPARAM(0));
    // hwnd.0 is *mut c_void; casting to usize gives us the address to carry through wParam
}

fn resolve_exe(hwnd: HWND) -> Option<String> {
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid == 0 {
        return None;
    }
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()? };
    let mut buf = vec![0u16; 1024];
    let mut size = buf.len() as u32;
    unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        )
        .ok()?
    };
    let path = String::from_utf16_lossy(&buf[..size as usize]);
    path.split('\\').last().map(|s| s.to_string())
}

pub fn run(
    config: Arc<Mutex<Config>>,
    tx: Sender<SwitcherEvent>,
    thread_id_tx: std::sync::mpsc::SyncSender<u32>,
) {
    let tid = unsafe { GetCurrentThreadId() };
    THREAD_ID.with(|cell| cell.set(tid));
    thread_id_tx.send(tid).ok();

    let hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(hook_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };

    let mut msg = MSG::default();
    loop {
        let result = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if result.0 <= 0 {
            break;
        }
        if msg.message == WM_APP {
            let hwnd = HWND(msg.wParam.0 as *mut core::ffi::c_void);
            if hwnd.0.is_null() {
                continue;
            }
            if let Some(exe) = resolve_exe(hwnd) {
                let (vid, pid, target_layer) = {
                    let cfg = config.lock().unwrap();
                    (cfg.vendor_id, cfg.product_id, cfg.layer_for(&exe))
                };
                if vid != 0 && pid != 0 {
                    hid::send_layer(pid, vid, target_layer).ok();
                }
                tx.send(SwitcherEvent {
                    current_exe: exe,
                    current_layer: target_layer,
                })
                .ok();
            }
        } else {
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    if !hook.0.is_null() {
        unsafe { UnhookWinEvent(hook) };
    }
}

pub fn stop(thread_id: u32) {
    unsafe {
        let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
    }
}
