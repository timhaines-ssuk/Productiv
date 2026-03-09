#[cfg(target_os = "windows")]
use chrono::Utc;

#[cfg(target_os = "windows")]
use super::runtime::WindowSnapshot;

#[cfg(target_os = "windows")]
pub(super) fn capture_window_snapshot() -> anyhow::Result<Option<WindowSnapshot>> {
    use std::{ffi::OsString, path::PathBuf};

    use windows::Win32::{
        Foundation::CloseHandle,
        System::{
            SystemInformation::GetTickCount64,
            Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
        },
        UI::{
            Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
            WindowsAndMessaging::{
                GetClassNameW, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
                GetWindowThreadProcessId,
            },
        },
    };

    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return Ok(None);
    }

    let mut title = vec![0u16; unsafe { GetWindowTextLengthW(hwnd) } as usize + 1];
    unsafe { GetWindowTextW(hwnd, &mut title) };
    let window_title = from_wide(&title);

    let mut class_name = vec![0u16; 256];
    let class_len = unsafe { GetClassNameW(hwnd, &mut class_name) };
    class_name.truncate(class_len as usize);
    let window_class = from_wide(&class_name);

    let mut process_id = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
    }
    let process_handle =
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) }?;
    let exe_path = get_process_path(process_handle);
    unsafe {
        let _ = CloseHandle(process_handle);
    }

    let process_name = exe_path
        .as_ref()
        .and_then(|path| PathBuf::from(path).file_name().map(|name| name.to_owned()))
        .map(OsString::from)
        .and_then(|name| name.into_string().ok())
        .unwrap_or_else(|| format!("pid-{process_id}"));

    let mut last_input = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        ..Default::default()
    };
    if !unsafe { GetLastInputInfo(&mut last_input) }.as_bool() {
        return Err(anyhow::anyhow!("GetLastInputInfo failed"));
    }
    let idle_seconds = (unsafe { GetTickCount64() } - last_input.dwTime as u64) / 1000;

    Ok(Some(WindowSnapshot {
        captured_at: Utc::now(),
        process_name,
        exe_path,
        window_title,
        window_class,
        idle_seconds,
    }))
}

#[cfg(target_os = "windows")]
fn get_process_path(handle: windows::Win32::Foundation::HANDLE) -> Option<String> {
    use windows::{
        Win32::System::Threading::{PROCESS_NAME_FORMAT, QueryFullProcessImageNameW},
        core::PWSTR,
    };

    let mut size = 1024u32;
    let mut buffer = vec![0u16; size as usize];
    let result = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
    };
    if result.is_err() {
        return None;
    }
    buffer.truncate(size as usize);
    Some(from_wide(&buffer))
}

#[cfg(target_os = "windows")]
fn from_wide(value: &[u16]) -> String {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};

    let first_zero = value.iter().position(|ch| *ch == 0).unwrap_or(value.len());
    OsString::from_wide(&value[..first_zero])
        .to_string_lossy()
        .trim()
        .to_owned()
}

#[cfg(not(target_os = "windows"))]
pub(super) fn capture_window_snapshot() -> anyhow::Result<Option<super::runtime::WindowSnapshot>> {
    Ok(None)
}
