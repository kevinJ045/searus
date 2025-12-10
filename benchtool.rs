use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 4 {
    eprintln!(
      "Usage: {} <exe_path> -p <parallel> -t <total_runs>",
      args[0]
    );
    return;
  }

  let exe_path = &args[1];
  let mut parallel = 1usize;
  let mut total_runs = 1usize;

  let mut i = 2;
  while i < args.len() {
    match args[i].as_str() {
      "-p" => {
        i += 1;
        parallel = args[i].parse().expect("Invalid number for -p");
      }
      "-t" => {
        i += 1;
        total_runs = args[i].parse().expect("Invalid number for -t");
      }
      _ => {}
    }
    i += 1;
  }

  println!(
    "Benchmarking '{}' with {} parallel threads, {} total runs",
    exe_path, parallel, total_runs
  );

  let start_time = Instant::now();

  let base_runs = total_runs / parallel;
  let remainder = total_runs % parallel;

  let timings = Arc::new(Mutex::new(Vec::with_capacity(total_runs)));
  let progress = Arc::new(Mutex::new(0usize));

  let mut handles = vec![];

  for thread_idx in 0..parallel {
    let exe_path = exe_path.clone();
    let timings = Arc::clone(&timings);
    let progress = Arc::clone(&progress);

    let runs_for_thread = if thread_idx < remainder {
      base_runs + 1
    } else {
      base_runs
    };

    let handle = thread::spawn(move || {
      for _ in 0..runs_for_thread {
        let run_start = Instant::now();

        let status = Command::new(&exe_path)
          .stdout(Stdio::null())
          .stderr(Stdio::null())
          .status()
          .expect("Failed to run executable");

        if !status.success() {
          eprintln!("Process exited with error: {:?}", status);
        }

        let duration = run_start.elapsed().as_secs_f64();
        {
          let mut t = timings.lock().unwrap();
          t.push(duration);
        }

        let mut prog = progress.lock().unwrap();
        *prog += 1;
        print!("\rProgress: {}/{}", *prog, total_runs);
        io::stdout().flush().unwrap();
      }
    });

    handles.push(handle);
  }

  for handle in handles {
    handle.join().unwrap();
  }

  let total_time = start_time.elapsed().as_secs_f64();
  let timings = timings.lock().unwrap();
  println!("\n\nResults over {} runs:", timings.len());

  let sum: f64 = timings.iter().sum();
  let avg = sum / timings.len() as f64;
  let min = timings.iter().cloned().fold(f64::INFINITY, f64::min);
  let max = timings.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

  println!("  Average: {:.6} s", avg);
  println!("  Min:     {:.6} s", min);
  println!("  Max:     {:.6} s", max);
  println!("Total time for all runs: {:.6} s", total_time);
}
