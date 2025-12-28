use crate::solve_puzzle::solve_puzzle;
use crate::solver_data::prepare_pieces_and_heuristics;
use env_logger::{Builder, Env};
use log::info;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use thousands::Separable;

mod bits;
mod board_order;
mod config;
mod pieces;
mod solve_puzzle;
mod solver_data;
mod structs;
mod util;

fn get_num_cores() -> usize {
    // Save one core to avoid grinding the system to a halt.
    match env::var("CORES") {
        Ok(value) => value.parse::<usize>().unwrap() - 1,
        Err(_e) => num_cpus::get() - 1,
    }
}

fn main() {
    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));
    builder.target(env_logger::Target::Stdout);
    builder.format_timestamp_millis();
    builder.init();

    let num_virtual_cores = get_num_cores();
    let overall_stopwatch = Instant::now();
    let max_depth = Arc::new(Mutex::new(0));
    let mut total_index_count: u64 = 0;
    let mut loop_count: u64 = 0;
    let index_counts = Arc::new(Mutex::new(vec![0u64; 257]));

    loop {
        loop_count += 1;

        let solver_data = Arc::new(prepare_pieces_and_heuristics());

        info!("Solving with {num_virtual_cores} cores...");

        let mut handles = vec![];

        for core in 0..num_virtual_cores {
            let max_depth = Arc::clone(&max_depth);
            let index_counts_clone = Arc::clone(&index_counts);
            let solver_data_clone = Arc::clone(&solver_data);

            let handle = std::thread::spawn(move || {
                for repeat in 1..6 {
                    info!("Core {core:02}: start loop {loop_count}, repeat {repeat}");
                    let stopwatch = Instant::now();
                    let solver_result = solve_puzzle(&solver_data_clone);

                    let mut counts = index_counts_clone.lock().unwrap();
                    for j in 0..257 {
                        counts[j] += solver_result.solve_indexes[j];
                    }
                    drop(counts);

                    {
                        let mut max_depth = max_depth.lock().unwrap();
                        if solver_result.max_depth > *max_depth {
                            *max_depth = solver_result.max_depth;
                        }
                    }

                    info!(
                        "Core {core:02}: finish loop {loop_count}, repeat {repeat}, best depth {} in {} seconds",
                        solver_result.max_depth,
                        stopwatch.elapsed().as_secs().separate_with_commas()
                    );
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        info!("Result");

        // This will only print valid numbers if you let the solver count how far you are.
        let index_counts_clone = index_counts.clone();
        let index_counts_locked = index_counts_clone.lock().unwrap();
        for ii in 0..=256 {
            let i: usize = ii as usize;
            if index_counts_locked[i] != 0 {
                println!("{i} {}", index_counts_locked[i].separate_with_commas());
            }
            total_index_count += index_counts_locked[i];
        }
        let elapsed_time_seconds = overall_stopwatch.elapsed().as_secs();
        let rate = total_index_count / elapsed_time_seconds;
        info!(
            "Total {} nodes in {} seconds, {} per second, max depth {}",
            total_index_count.separate_with_commas(),
            elapsed_time_seconds.separate_with_commas(),
            rate.separate_with_commas(),
            *max_depth.lock().unwrap()
        );
    }
}
