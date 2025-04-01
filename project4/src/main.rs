use std::env;
use std::fs::{read, read_to_string};
use std::process::exit;

struct Process {
    name: String,
    id: i8,
    arrival_time: i32,
    history: Vec<(char, i32)>,
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

fn main() {
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

    let mut i = 0;

    while i < input.len() {
        if input[i].contains("STOPHERE") {
            break;
        }

        let name_and_id: Vec<String> = input[i].split(' ').map(String::from).collect();
        i += 1;
        let history_strings: Vec<String> = input[i].split(' ').map(String::from).collect();

        let mut proc_history: Vec<(char, i32)> = Vec::new();
        let mut iter = history_strings.iter();

        while let Some(char_str) = iter.next() {
            let ch = char_str.chars().next().unwrap();

            if ch == 'N' {
                break;
            }

            println!("Current char is {}", ch);
            if let Some(num_str) = iter.next() {
                println!("Current int is {}", num_str);
                let num = num_str.parse::<i32>().unwrap();
                proc_history.push((ch, num));
            }
        }

        let proc = Process {
            name: name_and_id[0].clone(),
            id: (i as i8),
            arrival_time: name_and_id[1].parse().unwrap(),
            history: proc_history,
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
