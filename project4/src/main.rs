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
    active: Option<Process>,
    iactive: Option<Process>,
    oactive: Option<Process>,
    entryq: VecDeque<Process>,
    readyq: VecDeque<Process>,
    inputq: VecDeque<Process>,
    outputq: VecDeque<Process>,
    cpu_idle_status: bool,
    cpu_idle_time: u32,
    old_active_id: u32,
    total_terminated: u32,
    total_wait_time: u32,
}

#[derive(Debug, Clone)]
struct Process {
    name: String,
    id: u32,
    arrival_time: u32,
    history: Vec<(String, u32)>,
    history_index: usize,
    cpu_timer: u32,
    cpu_total: u32,
    cpu_burst_count: u32,
    io_timer: (u32, u32),
    io_total: (u32, u32),
    io_burst_count: (u32, u32),
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

    fn terminate(&self) {
        println!("Process {0} has ended.", self.id);
        println!("Name              {0}", self.name);
        println!("Started at time   {0} and ended at time {1}", self.arrival_time, self.end_time);
        println!("Total CPU time    {0} in {1} bursts", self.cpu_total, self.cpu_burst_count);
        println!("Total Input time  {0} in {1} bursts", self.io_total.0, self.io_burst_count.0);
        println!("Total Output time {0} in {1} bursts", self.io_total.1, self.io_burst_count.1);
        println!("Time waiting      {0}", self.wait_time);
    }
}

/*
* So the original implementation of this assignment, used
* nullptrs to do logic, as in, if the active program was
* nullptr, it knew that it had to grab another one.
*
* Current solution: program 0, would be named kernel, logic check ID / name
*******************************************************************************/
fn main() {
    const DEBUG_FLAG: bool = true;

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

    while input_index < input.len() {
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

    println!("Simulaation of CPU Scheduling");

    load_ready(&mut manager, 0);

    let mut timer: u32 = 0;

    while timer <= MAX_TIME {
        let mut copy = manager.clone();
        if timer % HOW_OFTEN == 0 {
            println!("Status at time {}", timer);

            if let Some(active_proc) = &manager.active {
                println!("Active is {}", active_proc.id);
                //active.debug_info();
            } else {
                println!("Active is 0");
            }

            if let Some(iactive_proc) = &manager.iactive {
                println!("IActive is {}", iactive_proc.id);
                //iactive.debug_info();
            } else {
                println!("IActive is 0");
            }

            if let Some(oactive_proc) = &manager.oactive {
                println!("OActive is {}", oactive_proc.id);
                //oactive.debug_info();
            } else {
                println!("OActive is 0");
            }

            dump_all_queues(copy);
        }

        process_active(&mut manager, timer);
        process_iactive(&mut manager, timer);
        process_oactive(&mut manager, timer);

        copy = manager.clone();

        if manager.entryq.is_empty() && get_total_processes(manager.clone()) == 0 {
            println!("The run has ended.");
            println!("The final value of the timer was: {}", timer);
            println!("The amount of time spent idle was: {}", manager.cpu_idle_time);
            println!("Number of terminated processes: {}", manager.total_terminated);
            let avg = manager.total_wait_time / manager.total_terminated;
            println!("The average waiting time for all terminated processes was: {}", avg);
            dump_all_queues(copy);
            return;
        }

        //println!("{:?}", manager);

        manager.old_active_id = 0;

        timer += 1;
    }
}

fn get_total_processes(manager: ProcessManager) -> usize {
    let mut amount = 0;

    amount += manager.readyq.len();
    amount += manager.inputq.len();
    amount += manager.outputq.len();

    if manager.active.is_some() {
        amount += 1;
    }

    if manager.iactive.is_some() {
        amount += 1;
    }

    if manager.oactive.is_some() {
        amount += 1;
    }

    amount
}

fn create_process(process_name: String, process_id: u32, arrival: u32, instructions: Vec<(String, u32)>) -> Process {
    Process {
        name: process_name,
        id: process_id,
        arrival_time: arrival,
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
fn load_ready(manager: &mut ProcessManager, total_proc: u32) {
    let free_space = IN_USE - total_proc;
    let mut added = 0;

    while added < free_space {
        if !manager.entryq.is_empty() {
            if let Some(proc) = manager.entryq.pop_front() {
                manager.readyq.push_back(proc);
                added += 1;
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
}

/* update_work_status
 *    checks if a process is at the end of its history, and terminates if so.
r*    if its not, updates the timers for the work to be done, puts it in
 *    the correct queue for processing, and prints out the change.
 *
 * Args
 *   proc - reference to process
 * ******************************************************************************/
fn update_work_status(manager: &mut ProcessManager, timer: u32, from: char) -> Option<Process> {
    let proc;

    let active_manager = if from == 'A' {
        &mut manager.active
    } else if from == 'I' {
        &mut manager.iactive
    } else {
        &mut manager.oactive
    };

    if let Some(tmp) = active_manager.as_mut() {
        proc = tmp;
    } else {
        panic!("Came from process_active, but the active process is not there!");
    }

    if proc.history_index == proc.history.len() - 1 {
        proc.end_time = timer + 1;
        proc.wait_time = (proc.end_time - proc.arrival_time) - proc.cpu_total - proc.io_total.0 - proc.io_total.1;
        proc.terminate();
        manager.total_terminated += 1;
        manager.total_wait_time += proc.wait_time;
    } else {
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
        if manager.readyq.is_empty() {
            load_ready(manager, get_total_processes(manager.clone()) as u32);
        }
        // Double check queue is not empty
        if !(manager.readyq.is_empty()) {
            if let Some(new_proc) = manager.readyq.pop_front() {
                manager.active = Some(new_proc);

                if let Some(ref mut active_proc) = manager.active.as_mut() {
                    active_proc.cpu_timer = active_proc.history[active_proc.history_index].1;
                }
            } else {
                manager.active = None;
            }
        } else if !(manager.entryq.is_empty() && ((get_total_processes(manager.clone()) as u32) < IN_USE)) {
            if let Some(new_proc) = manager.entryq.pop_front() {
                manager.active = Some(new_proc);

                if let Some(ref mut active_proc) = manager.active.as_mut() {
                    active_proc.cpu_timer = active_proc.history[active_proc.history_index].1;
                }
            } else {
                manager.active = None;
            }
        }
    }
    //
    // // Double check we have a process
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
        // If no process, add idle time
        manager.cpu_idle_time += 1;

        // Flag is so we only print once when we start idling
        if manager.cpu_idle_status {
            println!("At time {timer} Active is 0, so we idle for a while");
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
            if let Some(new_proc) = manager.inputq.pop_front() {
                manager.iactive = Some(new_proc);
            } else {
                return;
            }
        }
    }
    //
    // // Double check we have a process
    if manager.iactive.is_some() {
        if let Some(proc) = manager.iactive.as_mut() {
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
            if let Some(new_proc) = manager.outputq.pop_front() {
                manager.oactive = Some(new_proc);
            } else {
                return;
            }
        }
    }
    //
    // // Double check we have a process
    if manager.oactive.is_some() {
        if let Some(proc) = manager.oactive.as_mut() {
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
