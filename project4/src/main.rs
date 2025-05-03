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

#[rustfmt::skip]
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

#[rustfmt::skip]
fn main() {
    const DEBUG_FLAG: bool = true;

    // Get commandline arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Need an input file.");
        exit(1)
    }

    if DEBUG_FLAG { println!("PASSED ARGS"); }

    // Get lines of input
    let input: Vec<String> = read_to_string(&args[1])
        .unwrap()
        .lines()
        .map(String::from)
        .collect();

    let mut i = 0;

    while i < input.len() {
        if input[i].contains("STOPHERE") {
            if DEBUG_FLAG { println!("STOPHERE ENCOUNTERED"); }
            break;
        }
        if DEBUG_FLAG { println!("EXITED STOPHERE"); }

        // First line is program name and id
        let input_name_and_id: Vec<String> = input[i].split(' ').map(String::from).collect();
        let name_and_id: Vec<String> = input_name_and_id.into_iter().filter(|s| !s.is_empty()).collect();

        // Increment because
        i += 1;

        // Second line is program history
        let history_strings: Vec<String> = input[i].split(' ').map(String::from).collect();
        let history: Vec<String> = history_strings.into_iter().filter(|s| !s.is_empty()).collect();

        if DEBUG_FLAG { println!("STARTING PROCESS HISTORY STRING PROCESSING"); }

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

        if DEBUG_FLAG { println!("ATTEMPTING CREATE PROCESS STRUCT"); }

        // Create process based on the input strings we gathered
        let proc = Process {
            name: name_and_id[0].clone(),
            id: (i as i8),
            arrival_time: name_and_id[1].parse().unwrap(),
            history: instruction_set,
            history_index: 0,
            cpu_timer: 0,
            cpu_total: 0,
            cpu_burst_count: 0,
            io_timer: (0, 0),
            io_total: (0, 0),
            io_burst_count: (0, 0),
            end_time: 0,
            wait_time: 0,
        };
        proc.debug_info();
    }
}
