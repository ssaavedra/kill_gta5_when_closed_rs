mod processes;
use std::{thread, time};
use processes::{get_processes_by_name};

#[derive(Debug)]
enum ProgramStatus {
    StatusRunning,
    StatusDumb,
    StatusClosed,
}


fn main() {
    #[cfg(debug_assertions)]
    let debug = true;

    #[cfg(not(debug_assertions))]
    let debug = false;

    if debug {
        println!("Running in debug mode.");
    }

    let mut program_status = ProgramStatus::StatusClosed;

    loop {
        let mut inside_loop = false;
        get_processes_by_name("gta5.exe", None)
        .into_iter()
        .for_each(|item| {
            inside_loop = true;
            let item_window = item.get_main_window();

            #[cfg(debug_assertions)]
            let (pid, item_name, item_window_title) = (
                item.pid,
                item.name.clone(),
                item_window.map(|window| window.title()).flatten().unwrap_or(String::new()),
            );


            if item_window.is_some() {
                program_status = ProgramStatus::StatusRunning;
            } else {
                match program_status {
                    ProgramStatus::StatusRunning => {
                        program_status = ProgramStatus::StatusDumb;
                    },
                    ProgramStatus::StatusDumb => {
                        #[cfg(debug_assertions)]
                        println!("I would kill the process {}", pid);
                        #[cfg(not(debug_assertions))]
                        item.kill(None).ok();
                        program_status = ProgramStatus::StatusClosed;
                    },
                    ProgramStatus::StatusClosed  => {
                        //panic!("This should never happen!");
                    },
                }
            }
            
            #[cfg(debug_assertions)]
            println!("ITEM: {} ({}), has_window? {}, with title {}. Program status is {:?}", item_name, pid, item_window.is_some(), item_window_title, program_status);
        });
        if !inside_loop { program_status = ProgramStatus::StatusClosed; }

        if debug {
            thread::sleep(time::Duration::from_secs(1));
        } else {
            thread::sleep(time::Duration::from_secs(120));
        }
    }
}
