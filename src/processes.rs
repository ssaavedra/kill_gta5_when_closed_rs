use std::io;
use std::mem;
use std::mem::MaybeUninit;
use winapi::shared::minwindef::{DWORD, FALSE, HMODULE};
use winapi::um::psapi::{EnumProcessModules, GetModuleBaseNameA};
use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
use winapi::um::winnt::PROCESS_TERMINATE;
use winapi::um::winnt::PROCESS_VM_READ;
use winapi::um::winuser::SetLastErrorEx;
use windows_win::Process;
use windows_win::raw;

pub struct NamedProcess {
    pub name: String,
    pub pid: u32,
    _inner: Process,
}

pub struct Window {
    #[allow(dead_code)]
    h_wnd: winapi::shared::windef::HWND,
}


impl NamedProcess {
    pub fn open(pid: u32) -> io::Result<Self> {
        let process = Process::open(pid, PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_TERMINATE);

        process.map(|p| {
            let name = get_process_name(&p).ok().unwrap_or(String::new());
            NamedProcess {
                name,
                pid,
                _inner: p,
            }
        })
    }

    #[cfg(not(debug_assertions))]
    pub fn kill(self, code: Option<u32>) -> io::Result<()> {
        self._inner.terminate(code.unwrap_or(1))
    }
    
    pub fn get_main_window(&self) -> Option<Window> {
        let mut win_h = None;

        unsafe { SetLastErrorEx(0, 0) };
        raw::window::enum_by_until(None, |h_wnd| {
            let (pid, _tid) = raw::window::get_thread_process_id(h_wnd);
            if pid != self.pid {
                return 1
            }
            win_h = Some(h_wnd);
            0
        }).ok();
        unsafe { SetLastErrorEx(0, 0) };
        
        win_h.map(|h_wnd| Window { h_wnd })
    }

    #[cfg(debug_assertions)]
    pub fn get_main_window_title(&self) -> Option<String> {
        self.get_main_window().map(|w| {
            raw::window::get_text(w.h_wnd).ok()
        }).flatten()
    }

}


fn get_process_name(process: &Process) -> io::Result<String> {
    let mut module = MaybeUninit::<HMODULE>::uninit();
    let mut size = 0;
    if unsafe {
        EnumProcessModules(
            process.inner(),
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
            process.inner(),
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
