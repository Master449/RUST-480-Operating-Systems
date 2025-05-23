use std::process::{Command, Stdio};

const SPACESHIP_CAPACITY: u32 = 4;
const WIN_DISTANCE: u32 = 1000;
const STARTING_DISTANCE: u32 = 0;
const STARTING_FUEL: u32 = 300;
const DELAY: u32 = 1;

struct Spaceship {
    pid: u32,
    distance: u32,
    fuel: u32,
}

fn main() {
    let pipe_holder: Vec<(u32, u32)> = Vec::new();
    println!("Hello, world!");
}
