#![allow(dead_code)]

/**
 * CONTAINS CODE FROM https://github.com/DoumanAsh/windows-win-rs UNDER MIT LICENSE
 */
use std::io;

use winapi::shared::{minwindef::LPARAM, windef::HWND};
use winapi::um::winuser::{
    EnumChildWindows, EnumWindows, GetWindowTextW, GetWindowThreadProcessId, SetLastErrorEx,
};

///Retrieves window's title.
///
///# Parameters
///
///* ```window``` - A handle to the window to be tested.
///
///# Return
///
///* ```Ok``` - Contains name of class.
///* ```Err``` - Error reason.
pub fn get_text(window: HWND) -> io::Result<String> {
    const BUF_SIZE: usize = 512;
    let mut buff: [u16; BUF_SIZE] = [0; BUF_SIZE];

    let writ_chars = unsafe { GetWindowTextW(window, buff.as_mut_ptr(), BUF_SIZE as i32) };

    if writ_chars == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(String::from_utf16_lossy(&buff[0..writ_chars as usize]))
}

unsafe extern "system" fn callback_enum_windows_until<T: FnMut(HWND) -> i32>(
    window: HWND,
    param: LPARAM,
) -> i32 {
    let func = &mut *(param as *mut T);

    func(window)
}

///Enumerates over windows handles and calls callback on each
///
///# Note
/// Enumeration continues until callback return non-zero value.
///
///# Parameters
///
///* ```parent``` - Handle of parent window to look up through its children only. Optional.
///* ```cmp_func``` - Callback that will be called on each window.
///
///# Return
///
///* ```Ok``` - Success.
///* ```Err``` - Error reason.
pub fn enum_by_until<T: FnMut(HWND) -> i32>(
    parent: Option<HWND>,
    mut cmp_func: T,
) -> io::Result<()> {
    let lparam = &mut cmp_func as *mut _ as LPARAM;

    let result: i32;

    // Reset last error so that we catch errors from EnumWindows
    unsafe { SetLastErrorEx(0, 0) };
    if let Some(parent_window) = parent {
        result = unsafe {
            EnumChildWindows(
                parent_window,
                Some(callback_enum_windows_until::<T>),
                lparam,
            )
        };
    } else {
        result = unsafe { EnumWindows(Some(callback_enum_windows_until::<T>), lparam) };
    }

    // If cmp_func returns 0 then EnumWindows too.
    // But it is not an error case.
    if result == 0 {
        let error = io::Error::last_os_error();

        if let Some(errno) = error.raw_os_error() {
            if errno != 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }

    Ok(())
}

///Retrieves handle to a window by pid.
///
///# Parameters
///
///* ```pid``` - Pid of the process
///
///# Return
///
///* ```Ok``` - Success.
///* ```Err``` - Error reason.
pub fn get_by_pid(pid: u32) -> io::Result<Option<HWND>> {
    let mut found_window: Option<HWND> = None;

    let res = enum_by_until(None, |handle: HWND| {
        let (process_pid, _) = get_thread_process_id(handle);
        if process_pid == pid {
            found_window = Some(handle);
            return 0;
        }
        1
    });

    if let Err(error) = res {
        Err(error)
    } else {
        Ok(found_window)
    }
}

///Retrieves the identifier of the thread and process that created the specified window.
///
///# Parameters
///
///* ```window``` - Handle to a window.
///
///# Return(tuple)
///
///1. Process pid
///2. Thread id.
pub fn get_thread_process_id(window: HWND) -> (u32, u32) {
    let mut process_pid: u32 = 0;
    let thread_pid = unsafe { GetWindowThreadProcessId(window, &mut process_pid) };

    (process_pid, thread_pid)
}
