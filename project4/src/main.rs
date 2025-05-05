#![allow(dead_code)]
#![allow(unused_variables)]
use core::panic;
use std::collections::VecDeque;
use std::env;
use std::fs::read_to_string;
use std::process::exit;

struct QueueManager {
    entryq: VecDeque<Process>,
    readyq: VecDeque<Process>,
    inputq: VecDeque<Process>,
    outputq: VecDeque<Process>,
}

struct Process {
    name: String,
    id: i32,
    arrival_time: i32,
    history: Vec<(String, i32)>,
    history_index: usize,
    cpu_timer: i32,
    cpu_total: i32,
    cpu_burst_count: i32,
    io_timer: (i32, i32),
    io_total: (i32, i32),
    io_burst_count: (i32, i32),
    end_time: i32,
    wait_time: i32,
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
        println!("Total Input time  {0} in {1} bursts", self.io_timer.0, self.io_burst_count.0);
        println!("Total Output time {0} in {1} bursts", self.io_timer.1, self.io_burst_count.1);
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

    let mut current_id: i32 = 100;
    let mut input_index = 0;

    let mut allqueues = QueueManager {
        entryq: VecDeque::new(),
        readyq: VecDeque::new(),
        inputq: VecDeque::new(),
        outputq: VecDeque::new(),
    };

    // Get commandline arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Need an input file.");
        exit(1)
    }

    // Get lines of input
    let input: Vec<String> = read_to_string(&args[1])
        .unwrap()
        .lines()
        .map(String::from)
        .collect();

    while input_index < input.len() {
        if input[input_index].contains("STOPHERE  0") || input[input_index].contains("N 0") {
            break;
        }

        // First line is program name and id
        let name_and_id: Vec<String> = clean_and_split_string(input[input_index].clone());

        // Increment because we need 2 lines per loop
        input_index += 1;

        // Second line is program history
        let history: Vec<String> = clean_and_split_string(input[input_index].clone());

        // tmp variables for processing the instructions to
        // Vec<(Instruction, Burst) ...> format
        let mut instruction = "";
        let mut length;
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
                length = char_str.parse::<i32>().unwrap_or(0);
                instruction_set.push((instruction.to_string(), length));
                flip_flop = false;
            } else {
                // if the flipflop is false, we need the instruction
                instruction = char_str;
                flip_flop = true;
            }
        }

        // Increment ID number
        current_id += 1;

        let proc = create_process(name_and_id[0].clone(), current_id, name_and_id[1].parse().unwrap(), instruction_set);

        // Create process based on the input strings we gathered
        //proc.debug_info();

        allqueues.entryq.push_back(proc);

        // Increment again
        input_index += 1;
    }
    dump_all_queues(allqueues);
}

fn create_process(process_name: String, process_id: i32, arrival: i32, instructions: Vec<(String, i32)>) -> Process {
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

fn kernel_takeover() -> Process {
    create_process("KERNEL".to_string(), 0, 0, Vec::new())
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
 *
 * Args
 *   q    - reference to a queue
 *   name - name of the queue to print
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
 *
 * Args
 *   queues - QueueManager struct
 * **************************************************/
fn dump_all_queues(queues: QueueManager) {
    dump_queue(queues.entryq, "Entry".to_string());
    dump_queue(queues.readyq, "Ready".to_string());
    dump_queue(queues.inputq, "Input".to_string());
    dump_queue(queues.outputq, "Output".to_string());
}

/* update_work_status
 *    checks if a process is at the end of its history, and terminates if so.
 *    if its not, updates the timers for the work to be done, puts it in
 *    the correct queue for processing, and prints out the change.
 *
 * Args
 *   proc - reference to process
 * ******************************************************************************/
fn update_work_status<'a>(all: &mut Vec<VecDeque<&'a Process>>, proc: &'a mut Process, timer: i32) -> Process {
    if proc.history_index == proc.history.len() - 1 {
        proc.end_time = timer + 1;
        proc.wait_time = (proc.end_time - proc.arrival_time) - proc.cpu_total - proc.io_total.0 - proc.io_total.1;
        proc.terminate();
        // TODO: Figure out an alterative to nullptr move arounds
    } else {
        proc.history_index += 1;
        let new_task = proc.history[proc.history_index].0.clone();

        match new_task.as_str() {
            "I" => {
                proc.io_timer.0 = proc.history[proc.history_index].1;
                all[2].push_back(&*proc);
            }
            "O" => {
                proc.io_timer.1 = proc.history[proc.history_index].1;
                all[2].push_back(&*proc);
            }
            "C" => {
                proc.cpu_timer = proc.history[proc.history_index].1;
                all[2].push_back(&*proc);
            }
            _ => {
                // uh oh
                panic!("HUH??? WHA ??? ?? ?");
            }
        }
    }

    // Return control to the kernel
    kernel_takeover()
}

/* check_num_process
 *    checks to see if there is room to add any processes
 *    from the entryq to the readyq
 * ***************************************************************/
fn check_num_process() {}

/* process_active
 *    processes the CPUs bursts. If the burst reaches 0
 *    sends it to update_work_status to see if it
 *    stops or moves onto bigger and better things
 *
 *    if nothing is here it adds idle time to the total
 * ***************************************************************/
fn process_active(proc: &mut Process) {}

/* process_iactive
 *    processes the input burst. If the burst reaches 0
 *    sends it to update_work_status to see if it
 *    stops or moves onto bigger and better things
 *
 * ***************************************************************/
fn process_iactive(proc: &mut Process) {}

/* process_oactive
 *    processes the output burst. If the burst reaches 0
 *    sends it to update_work_status to see if it
 *    stops or moves onto bigger and better things
 *
 * ***************************************************************/
fn process_oactive(proc: &mut Process) {}
