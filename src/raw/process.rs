#![allow(dead_code)]
/**
 * CONTAINS CODE FROM https://github.com/DoumanAsh/windows-win-rs UNDER MIT LICENSE
 */
 
use std::io;

use winapi::ctypes::c_uint;
use winapi::shared::ntdef::HANDLE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};

///Opens process by pid.
///
///# Note:
///See information about access rights:
///https://msdn.microsoft.com/en-us/library/windows/desktop/ms684880%28v=vs.85%29.aspx
///
///# Parameters
///
///* ```pid``` - Pid of the process.
///* ```access_rights``` - Bit mask that specifies desired access rights.
///
///# Return
///
///* ```Ok``` - Handle to opened process.
///* ```Err``` - Error reason.
pub fn open(pid: u32, access_rights: u32) -> io::Result<HANDLE> {
    let result = unsafe { OpenProcess(access_rights, 0, pid) };

    if result.is_null() {
        return Err(io::Error::last_os_error());
    }

    Ok(result)
}

///Closes opened process.
///
///# Parameters
///
///* ```process``` - pointer to a opened process.
///
///# Return
///
///* ```Ok``` - Success.
///* ```Err``` - Error reason.
pub fn close(process: HANDLE) -> io::Result<()> {
    let result = unsafe { CloseHandle(process) };

    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

///Terminates process.
///
///# Parameters
///
///* ```process``` - Pointer to a opened process.
///* ```code``` - Exit code that shall be used by affected process.
///
///# Note:
///
///It prevents process from running any clean-up.
pub fn terminate(process: HANDLE, code: c_uint) -> io::Result<()> {
    if unsafe { TerminateProcess(process, code) } != 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
