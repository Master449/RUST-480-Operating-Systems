use std::collections::VecDeque;
use std::env;
use std::fs::{read, read_to_string};
use std::process::exit;

struct Process {
    name: String,
    id: i8,
    arrival_time: i32,
    history: Vec<(String, i32)>,
    history_index: i8,
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
        print!("\n");

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
}

fn main() {
    const DEBUG_FLAG: bool = true;

    // Get commandline arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Need an input file.");
        exit(1)
    }

    if DEBUG_FLAG {
        println!("PASSED ARGS");
    }

    // Get lines of input
    let input: Vec<String> = read_to_string(&args[1])
        .unwrap()
        .lines()
        .map(String::from)
        .collect();

    let mut i = 0;

    while i < input.len() {
        if input[i].contains("STOPHERE") {
            if DEBUG_FLAG {
                println!("STOPHERE ENCOUNTERED");
            }
            break;
        }
        if DEBUG_FLAG {
            println!("EXITED STOPHERE");
        }

        // First line is program name and id
        let input_name_and_id: Vec<String> = input[i].split(' ').map(String::from).collect();
        let name_and_id: Vec<String> = input_name_and_id
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();

        // Increment because
        i += 1;

        // Second line is program history
        let history_strings: Vec<String> = input[i].split(' ').map(String::from).collect();
        let history: Vec<String> = history_strings
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();

        if DEBUG_FLAG {
            println!("STARTING PROCESS HISTORY STRING PROCESSING");
        }

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
            if char_str == "N" {
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

        if DEBUG_FLAG {
            println!("ATTEMPTING CREATE PROCESS STRUCT");
        }

        let proc = create_process(name_and_id[0].clone(), i as i8, name_and_id[1].parse().unwrap(), instruction_set);
        // Create process based on the input strings we gathered
        proc.debug_info();
    }
}

#[rustfmt::skip]
fn create_process(process_name: String, process_id: i8, arrival: i32, instructions: Vec<(String, i32)>) -> Process {
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

/* dump_queue
 *    takes a reference to a queue and prints all the process
 *    IDs that are inside. Mainly for the summaries.
 *
 * Args
 *   q    - reference to a queue
 *   name - name of the queue to print
 ****************************************************************/
fn dump_queue(q: &VecDeque<Process>, name: String) {
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
 *   all - reference Vec holding all VecDeques
 * **************************************************/
fn dump_all_queues(all: &Vec<VecDeque<Process>>) {
    let names = ["Entry", "Ready", "Input", "Output"];
    for i in 1..4 {
        dump_queue(&all[i], names[i].to_string());
    }
}

/* update_work_status
 *    checks if a process is at the end of its history, and terminates if so.
 *    if its not, updates the timers for the work to be done, puts it in
 *    the correct queue for processing, and prints out the change.
 *
 * Args
 *   proc - reference to process
 * ******************************************************************************/
fn update_work_status(proc: &mut Process, timer: i32) {
    if (proc.history_index as usize) == proc.history.len() - 1 {
        proc.end_time = timer + 1;
        proc.wait_time = (proc.end_time - proc.arrival_time) - proc.cpu_total - proc.io_total.0 - proc.io_total.1;
    }
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
