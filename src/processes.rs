use std::io;
use std::mem;
use std::mem::MaybeUninit;
use std::ptr;
use winapi::shared::minwindef::{DWORD, FALSE, HMODULE};
use winapi::um::psapi::{EnumProcessModules, GetModuleBaseNameA};
use winapi::um::winnt::HANDLE;
use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
use winapi::um::winnt::PROCESS_TERMINATE;
use winapi::um::winnt::PROCESS_VM_READ;
use crate::raw;

pub struct NamedProcess {
    pub name: String,
    pub pid: u32,
    handle: HANDLE,
}

#[derive(Clone, Copy)]
pub struct Window {
    #[allow(dead_code)]
    h_wnd: winapi::shared::windef::HWND,
}

impl Window {
    #[cfg(debug_assertions)]
    pub fn title(&self) -> io::Result<String> {
        raw::window::get_text(self.h_wnd)
    }
}


impl NamedProcess {
    pub fn open(pid: u32) -> io::Result<Self> {
        let access_rights = PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_TERMINATE;

        return match raw::process::open(pid, access_rights) {
            Ok(handle) => {
                let name = Self::get_name(handle).ok().unwrap_or(String::new());
                Ok(NamedProcess {
                        name,
                        pid,
                        handle,
                })
            },
            Err(error) => Err(error),
        };
    }

    pub fn close(&mut self) {
        if !self.handle.is_null() {
            raw::process::close(self.handle).expect("Unable to close process");
            self.handle = ptr::null_mut();
        }
    }
        
    fn get_name(process_handle: HANDLE) -> io::Result<String> {
        let mut module = MaybeUninit::<HMODULE>::uninit();
        let mut size = 0;
        if unsafe {
            EnumProcessModules(
                process_handle,
                module.as_mut_ptr(),
                mem::size_of::<HMODULE>() as u32,
                &mut size,
            )
        } == FALSE
        {
            return Err(io::Error::last_os_error());
        }

        let module = unsafe { module.assume_init() };
        
        let mut buffer = Vec::<u8>::with_capacity(64);
        let length = unsafe {
            GetModuleBaseNameA(
                process_handle,
                module,
                buffer.as_mut_ptr().cast(),
                buffer.capacity() as u32,
            )
        };

        if length == 0 {
            return Err(io::Error::last_os_error());
        }

        unsafe { buffer.set_len(length as usize) };
        
        Ok(String::from_utf8(buffer).unwrap())
    }


    #[cfg(not(debug_assertions))]
    pub fn kill(self, code: Option<u32>) -> io::Result<()> {
        raw::process::terminate(self.handle, code.unwrap_or(1)).map(|_| {
            mem::drop(self);
        })
    }

    pub fn get_main_window(&self) -> Option<Window> {
        raw::window::get_by_pid(self.pid).ok().flatten().map(|h_wnd| Window { h_wnd })
    }
}

impl Drop for NamedProcess {
    fn drop(&mut self) {
        self.close()
    }
}

pub fn enum_proc () -> io::Result<Vec<u32>> {
    let mut pids = Vec::<DWORD>::with_capacity(1024);
    let mut size = 0;
    // Safety: the pointer is valid and the size matches the capacity
    if unsafe {
        winapi::um::psapi::EnumProcesses(
            pids.as_mut_ptr(),
            (pids.capacity() * mem::size_of::<DWORD>()) as u32,
            &mut size,
        ) 
    } == FALSE
    {
        return Err(io::Error::last_os_error());
    }

    let count = size as usize / mem::size_of::<DWORD>();
    // Safety: the call succeeded and count equals right amount of items
    unsafe { pids.set_len(count) };

    Ok(pids)
}


pub fn get_processes_by_name(name: &str, initial_capacity: Option<usize>) -> Vec<NamedProcess> {
    let mut ps = Vec::<NamedProcess>::with_capacity(initial_capacity.unwrap_or(5));
    enum_proc()
    .unwrap()
    .into_iter()
    .for_each(|pid| match NamedProcess::open(pid) {
        Ok(proc) => {
            if proc.name.to_lowercase().contains(name) {
                ps.push(proc);
            }
        },
        Err(_) => {},
    });

    ps
}
