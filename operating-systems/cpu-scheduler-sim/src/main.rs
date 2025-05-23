#![allow(dead_code)]
#![allow(unused_variables)]
use core::panic;
use std::collections::VecDeque;
use std::env;
use std::fs::read_to_string;
use std::process::exit;

const IN_USE: u32 = 5;
const MAX_TIME: u32 = 500;
const HOW_OFTEN: u32 = 25;

#[derive(Debug, Clone)]
struct ProcessManager {
    // Currently active and I/O active processes
    active: Option<Process>,
    iactive: Option<Process>,
    oactive: Option<Process>,

    // The various queues
    entryq: VecDeque<Process>,
    readyq: VecDeque<Process>,
    inputq: VecDeque<Process>,
    outputq: VecDeque<Process>,

    // Stats about the system
    cpu_idle_status: bool,
    cpu_idle_time: u32,
    total_terminated: u32,
    total_wait_time: u32,

    // This is a failsafe to make sure
    // that a process that has gone from
    // active to I/O, can't two two work
    // cycles in 1 CPU cycle
    old_active_id: u32,
}

#[derive(Debug, Clone)]
struct Process {
    // Indentifying information
    name: String,
    id: u32,

    // When the process arrived
    // versus when it started work
    arrival_time: u32,
    start_time: u32,

    // The instruction queue along with index
    history: Vec<(String, u32)>,
    history_index: usize,

    // CPU stats: Work timer, total cycles, and total bursts
    cpu_timer: u32,
    cpu_total: u32,
    cpu_burst_count: u32,

    // I/O Stats: Timer, Total work cycles, total bursts
    io_timer: (u32, u32),
    io_total: (u32, u32),
    io_burst_count: (u32, u32),

    // Wait time is total time spent
    // waiting in queues
    end_time: u32,
    wait_time: u32,
}

impl Process {
    fn debug_info(&self) {
        println!("Name:          {}", self.name);
        println!("ID:            {}", self.id);
        println!("Arrival:       {}", self.arrival_time);

        print!("History:       ");
        for it in self.history.clone() {
            print!("({}, {}) ", it.0, it.1);
        }
        println!();

        println!("History Idx:   {}", self.history_index);
        println!("CPU Timer:     {}", self.cpu_timer);
        println!("CPU Total:     {}", self.cpu_total);
        println!("CPU Bursts:    {}", self.cpu_burst_count);
        println!("IO Timer:      ({}, {})", self.io_timer.0, self.io_timer.1);
        println!("IO Total:      ({}, {})", self.io_total.0, self.io_total.1);
        println!("IO Bursts:     ({}, {})", self.io_burst_count.0, self.io_burst_count.1);
        println!("End Time:      {}", self.end_time);
        println!("Waiting Time:  {}", self.wait_time);
    }

    // terminate function, just prints out a summary of the process on exit
    fn terminate(&self) {
        println!("Process {0} has ended.", self.id);
        println!("Name              {0}", self.name);
        println!("Started at time   {0} and ended at time {1}", self.start_time, self.end_time);
        println!("Total CPU time    {0} in {1} bursts", self.cpu_total, self.cpu_burst_count);
        println!("Total Input Time  {0} in {1} bursts", self.io_total.0, self.io_burst_count.0);
        println!("Total Output Time {0} in {1} bursts", self.io_total.1, self.io_burst_count.1);
        println!("Time waiting      {0}\n", self.wait_time);
    }
}

#[rustfmt::skip]
fn main() {
    let mut current_id: u32 = 100;
    let mut input_index = 0;
    let mut input: Vec<String>;

    let mut manager = ProcessManager {
        active: None,
        iactive: None,
        oactive: None,
        entryq: VecDeque::new(),
        readyq: VecDeque::new(),
        inputq: VecDeque::new(),
        outputq: VecDeque::new(),
        cpu_idle_status: false,
        cpu_idle_time: 0,
        old_active_id: 0,
        total_terminated: 0,
        total_wait_time: 0,
    };

    // Get commandline arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Need an input file.");
        exit(1)
    }

    // Get lines of input
    if let Ok(success) = read_to_string(&args[1]) {
        input = success.lines().map(String::from).collect();
    } else {
        panic!("Could not transform `input` instructions from input file into Vec<String>");
    }

    // Loop through input lines
    while input_index < input.len() {
        // Check for delimiter
        if input[input_index].contains("STOPHERE  0") || input[input_index].contains("N 0") {
            break;
        }

        // First line is program name and id
        let name_and_id: Vec<String> = clean_and_split_string(input[input_index].clone());

        // Second line is program history
        let history: Vec<String> = clean_and_split_string(input[input_index + 1].clone());

        // tmp variables for processing the instructions to
        // Vec<(Instruction, Burst) ...> format
        let mut instruction = "";
        let mut length: u32;
        let mut instruction_set = Vec::new();
        let mut flip_flop = false;

        let iter = history.iter();

        // Each instruction comes in a pair, we are going to flip flop
        // back and fourth to get the instruction, then get the burst
        for char_str in iter {
            // If the instructions start with N, its cutoff time
            if char_str == "N" || char_str == "STOPHERE" {
                break;
            }

            if flip_flop {
                // if flip flop is true, then we have the instruction already
                // and just need to get the burst time
                if let Ok(length) = char_str.parse::<u32>() {
                    instruction_set.push((instruction.to_string(), length));
                    flip_flop = false;
                } else {
                    panic!("Could not parse char as u32 inside of flip_flop");
                }
            } else {
                // if the flipflop is false, we need the instruction
                instruction = char_str;
                flip_flop = true;
            }
        }

        // Increment ID number
        current_id += 1;

        // Create process based on the input strings we gathered
        if let Ok(arrival_time) = name_and_id[1].parse() {
            let proc = create_process(name_and_id[0].clone(), current_id, arrival_time, instruction_set);

            // Push it to entryq
            manager.entryq.push_back(proc);
        }

        // Increment again
        input_index += 2;
    }

    // Start simulation, setting timer to 0, and loading ready queue
    println!("Simulation of CPU Scheduling\n");

    let mut timer: u32 = 0;

    load_ready_from_entry(&mut manager, 0, timer);

    // Main simulation loop
    while timer <= MAX_TIME {

        // If we meet the summary printing time, print the summary
        if timer % HOW_OFTEN == 0 {
            let copy = manager.clone();

            // Print the status of each process, active or not
            println!("Status at time {}", timer);
            println!("Active is {}", manager.clone().active.map(|active| active.id).unwrap_or(0));
            println!("IActive is {}", manager.clone().iactive.map(|iactive| iactive.id).unwrap_or(0));
            println!("OActive is {}", manager.clone().oactive.map(|oactive| oactive.id).unwrap_or(0));

            // Print out the contents of all queues
            dump_all_queues(copy);
        }

        // Process the various cycles
        process_active(&mut manager, timer);
        process_iactive(&mut manager, timer);
        process_oactive(&mut manager, timer);

        // If the program has ended, print a full run summary
        if manager.entryq.is_empty() && get_total_processes(manager.clone()) == 0 {
            let copy = manager.clone();
            println!("The run has ended.");
            println!("The final value of the timer was: {}", timer);
            println!("The amount of time spent idle was: {}", manager.cpu_idle_time);
            println!("Number of terminated processes: {}", manager.total_terminated);
            println!("The average waiting time for all terminated processes was: {}", manager.total_wait_time / manager.total_terminated);
            dump_all_queues(copy);
            return;
        }

        // Reset the active id flag, and increment timer
        manager.old_active_id = 0;
        timer += 1;
    }
}

#[rustfmt::skip]
fn get_total_processes(manager: ProcessManager) -> usize {
    let mut amount = 0;

    amount += manager.readyq.len() 
            + manager.inputq.len() 
            + manager.outputq.len()
            + (manager.active.is_some() as usize)
            + (manager.iactive.is_some() as usize)
            + (manager.oactive.is_some() as usize);
    amount
}

// Just a helper function since a lot of these are initalized to 0 anyway
fn create_process(process_name: String, process_id: u32, arrival: u32, instructions: Vec<(String, u32)>) -> Process {
    Process {
        name: process_name,
        id: process_id,
        arrival_time: arrival,
        start_time: 0,
        history: instructions,
        history_index: 0,
        cpu_timer: 0,
        cpu_total: 0,
        cpu_burst_count: 0,
        io_timer: (0, 0),
        io_total: (0, 0),
        io_burst_count: (0, 0),
        end_time: 0,
        wait_time: 0,
    }
}

#[rustfmt::skip]
fn load_ready_from_entry(manager: &mut ProcessManager, total_proc: u32, timer: u32) {
    // Temporary variables
    let free_space = IN_USE - total_proc;
    let mut added = 0;

    // While we are less than capacity
    while added < free_space {
        if !manager.entryq.is_empty() {
            // If entryq is not empty, get one from there
            if let Some(proc) = manager.entryq.pop_front().as_mut() {
                // If its arrival is past timer, update its start time
                // and push it to the ready queue
                if proc.arrival_time <= timer {
                    proc.start_time = timer;
                    manager.readyq.push_back(proc.clone());
                    added += 1;
                    println!("Process {} has moved from the Entry Queue into the Ready Queue at time {}\n", proc.id, timer);
                }
            }
            added += 1;
         } else {
            return
        }
    }
}

// Takes a string, splits it by whitespaces and then
// removes the whitespaces from the vec
fn clean_and_split_string(raw: String) -> Vec<String> {
    let split_spaces: Vec<String> = raw.split(' ').map(String::from).collect();
    let removed_whitespaces: Vec<String> = split_spaces.into_iter().filter(|s| !s.is_empty()).collect();
    removed_whitespaces
}

/* dump_queue
 *    takes a reference to a queue and prints all the process
 *    IDs that are inside. Mainly for the summaries.
 ****************************************************************/
fn dump_queue(q: VecDeque<Process>, name: String) {
    print!("{name} Queue Contents: ");

    if q.is_empty() {
        print!("(Empty)");
    } else {
        for item in q {
            print!("{0} ", item.id);
        }
    }

    println!();
}

/* dump_all_queues
 *    Just prints out every queue sequentially
 * **************************************************/
fn dump_all_queues(manager: ProcessManager) {
    dump_queue(manager.entryq, "Entry".to_string());
    dump_queue(manager.readyq, "Ready".to_string());
    dump_queue(manager.inputq, "Input".to_string());
    dump_queue(manager.outputq, "Output".to_string());
    println!();
}

/* update_work_status
*    checks if a process is at the end of its history, and terminates if so.
*    if its not, updates the timers for the work to be done, puts it in
*    the correct queue for processing, and prints out the change.
*
*    @Args
*
******************************************************************************/
fn update_work_status(manager: &mut ProcessManager, timer: u32, from: char) -> Option<Process> {
    let proc;

    // Change which one is getting moved based on where the caller came from
    let active_manager = if from == 'A' {
        &mut manager.active
    } else if from == 'I' {
        &mut manager.iactive
    } else {
        &mut manager.oactive
    };

    // Check to make sure the process exists again, as it is an Option<_>
    if let Some(tmp) = active_manager.as_mut() {
        proc = tmp;
    } else {
        panic!("Came from {}, but the active process is not there!", from);
    }

    // If we are at the end of the programs history
    if proc.history_index == proc.history.len() - 1 {
        // Update the processes and manager stats
        proc.end_time = timer + 1;
        proc.wait_time = (proc.end_time - proc.start_time) - proc.cpu_total - proc.io_total.0 - proc.io_total.1;
        manager.total_terminated += 1;
        manager.total_wait_time += proc.wait_time;

        // Terminate
        proc.terminate();
    } else {
        // Otherwise it needs a new place to go
        proc.history_index += 1;
        let new_task = proc.history[proc.history_index].0.clone();

        match new_task.as_str() {
            "I" => {
                proc.io_timer.0 = proc.history[proc.history_index].1;
                manager.inputq.push_back(proc.clone());
            }
            "O" => {
                proc.io_timer.1 = proc.history[proc.history_index].1;
                manager.outputq.push_back(proc.clone());
            }
            "C" => {
                proc.cpu_timer = proc.history[proc.history_index].1;
                manager.readyq.push_back(proc.clone());
            }
            _ => {
                // uh oh
                panic!("update_work_status: new_task is not one of 'I', 'O', or 'C'.");
            }
        }
    }

    // Return none, as the return will take the place of the origin caller process
    None
}

/* process_active
 *    processes the CPUs bursts. If the burst reaches 0
 *    sends it to update_work_status to see if it
 *    stops or moves onto bigger and better things
 *
 *    if nothing is here it adds idle time to the total
 * ***************************************************************/
#[rustfmt::skip]
fn process_active(manager: &mut ProcessManager, timer: u32) {
    // If no process, see if we can load one
    if manager.active.is_none() {
        
        //Check if readyq is empty, if so, attempt to load it
        if manager.readyq.is_empty() {
            load_ready_from_entry(manager, get_total_processes(manager.clone()) as u32, timer);
        }

        // Double check queue is not empty
        if !(manager.readyq.is_empty()) {
            // If a program exists on the readyq, pop it
            // make it active and update its work index
            if let Some(new_proc) = manager.readyq.pop_front() {
                manager.active = Some(new_proc);

                if let Some(ref mut active_proc) = manager.active.as_mut() {
                    active_proc.cpu_timer = active_proc.history[active_proc.history_index].1;
                }
            } else {
                manager.active = None;
            }
        } else if !(manager.entryq.is_empty() && ((get_total_processes(manager.clone()) as u32) < IN_USE)) {
            // If the ready queue is still empty for some reason, grab one from entry queue
            if let Some(new_proc) = manager.entryq.pop_front() {
                manager.active = Some(new_proc);

                // Update its work index
                if let Some(ref mut active_proc) = manager.active.as_mut() {
                    active_proc.cpu_timer = active_proc.history[active_proc.history_index].1;
                }
            }
        }
    }

    // Double check we have a process
    if manager.active.is_some() {
        if let Some(proc) = manager.active.as_mut() {
            // If we've done some work, idle flag gets reset
            manager.cpu_idle_status = true;

            // Increment cpu total, and decrement work timer
            proc.cpu_total += 1;
            proc.cpu_timer -= 1;

            // If we are at the end of this burst
            // add it to total, and see where next
            if proc.cpu_timer == 0 {
                proc.cpu_burst_count += 1;
                manager.old_active_id = proc.id;
                manager.active = update_work_status(manager, timer, 'A');
            }
        }
    } else {
        // If no process, and ready and entry queue are empty
        // we sit and wait for the simulation to end, or for
        // I/O to put their process back into the ready queue
        manager.cpu_idle_time += 1;

        // Flag is so we only print once when we start idling
        if manager.cpu_idle_status {
            println!("At time {timer} Active is 0, so we idle for a while\n");
            manager.cpu_idle_status = false;
        }
    }
}

/* process_iactive
 *    processes the input burst. If the burst reaches 0
 *    sends it to update_work_status to see if it
 *    stops or moves onto bigger and better things
 *
 * ***************************************************************/
fn process_iactive(manager: &mut ProcessManager, timer: u32) {
    // If no process, see if we can load one
    if manager.iactive.is_none() {
        // Double check queue is not empty
        if !(manager.inputq.is_empty()) {
            // If theres one to grab, great! Otherwise exit
            if let Some(new_proc) = manager.inputq.pop_front() {
                manager.iactive = Some(new_proc);
            } else {
                return;
            }
        }
    }

    // Double check we have a process
    if manager.iactive.is_some() {
        if let Some(proc) = manager.iactive.as_mut() {
            // Double check for double dip flag
            if manager.old_active_id == proc.id {
                return;
            }

            // Increment cpu total, and decrement work timer
            proc.io_total.0 += 1;
            proc.io_timer.0 -= 1;

            // If we are at the end of this burst
            // add it to total, and see where next
            if proc.io_timer.0 == 0 {
                proc.io_burst_count.0 += 1;
                manager.iactive = update_work_status(manager, timer, 'I');
            }
        }
    }
}

/* process_oactive
 *    processes the output burst. If the burst reaches 0
 *    sends it to update_work_status to see if it
 *    stops or moves onto bigger and better things
 *
 * ***************************************************************/
fn process_oactive(manager: &mut ProcessManager, timer: u32) {
    // If no process, see if we can load one
    if manager.oactive.is_none() {
        // Double check queue is not empty
        if !(manager.outputq.is_empty()) {
            // If theres one to grab, great! Otherwise exit
            if let Some(new_proc) = manager.outputq.pop_front() {
                manager.oactive = Some(new_proc);
            } else {
                return;
            }
        }
    }

    // Double check we have a process
    if manager.oactive.is_some() {
        if let Some(proc) = manager.oactive.as_mut() {
            // Double check for double dip flag
            if manager.old_active_id == proc.id {
                return;
            }

            // Increment cpu total, and decrement work timer
            proc.io_total.1 += 1;
            proc.io_timer.1 -= 1;

            // If we are at the end of this burst
            // add it to total, and see where next
            if proc.io_timer.1 == 0 {
                proc.io_burst_count.1 += 1;
                manager.oactive = update_work_status(manager, timer, 'O');
            }
        }
    }
}
