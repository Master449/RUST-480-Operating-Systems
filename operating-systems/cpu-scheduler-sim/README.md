# Project 4: CPU Scheduling Simulation

Project 4 for 480 was to create a FCFS CPU Scheduling simulation. I enjoyed this assignment quite a bit and decided it was going to be a good way to learn how to learn Rust. This is the first one I completed.

As of right now (5/9/2025):
- 2/3 diffs are clean (tiny.txt and small.txt)
- input.txt diff seems to mostly be timing differences (there was timing differences from the original professor example as well)
- input.txt still has some artifacts though: a couple of programs have strange timings attached to start_time or end_time

# Program Flow

- Read in program data from input file
- Create ProcessManager to manage Ready, Entry, Input and Output queues
- Repeat as needed:
  - Parse program data into Process struct
  - Push program into Entry Queue
- Load Ready Queue with maximum programs allowed to run at once
- Start Simulation, Repeating as needed:
  - Process Active CPU program
    - Load program if current active does not exist
    - Decrement CPU timer
    - Increase total CPU burst time 
    - If done, Update Work Status
  - Process Active Input program
    - If no program, grab one off the queue
    - Decrement Input timer
    - Increase total input burst time 
    - If done, Update Work Status
  - Process Active Output program
    - If no program, grab one off the queue
    - Decrement Output timer
    - Increase total Output burst time 
    - If done, Update Work Status
- Update Work Status
  - If program has no other work -> Terminate it, printing out a summary
  - Otherwise, move the history index forward
  - Update the proper work timer (CPU, I/O)
  - Move the process to the proper queue
- Every 25 (configurable) timer cycles, print out a summary showing the queue contents
- If all queues are empty, and no program is active for CPU, I or O, exit and print run summary

