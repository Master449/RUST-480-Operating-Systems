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
            print!("({}, {})", it.0, it.1);
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
    println!("Hello, world!");

    let proc = Process {
        name: String::from("STARWAR"),
        id: 0,
        arrival_time: 0,
        history: vec![('C', 4), ('I', 20), ('O', 14)],
        history_index: 0,
        cpu_timer: 0,
        cpu_total: 120,
        cpu_burst_count: 27,
        io_timer: (20, 45),
        io_total: (4, 6),
        io_burst_count: (2, 1),
        end_time: 50,
        wait_time: 2,
    };

    proc.debug_info();
}
