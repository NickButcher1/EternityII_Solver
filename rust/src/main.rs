use crate::config::{MAX_HEURISTIC_INDEX, MAX_NODE_COUNT, MIN_SOLVE_INDEX_TO_SAVE};
use crate::solver_data::{prepare_pieces_and_heuristics, SolverData};
use crate::structs::{RotatedPiece, SolverResult};
use env_logger::{Builder, Env};
use log::info;
use rand::Rng;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use thousands::Separable;

mod board_order;
mod config;
mod pieces;
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

    loop {
        loop_count += 1;

        let solver_data = Arc::new(prepare_pieces_and_heuristics());

        info!("Solving with {num_virtual_cores} cores...");

        let index_counts = Arc::new(Mutex::new(vec![0u64; 257]));

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

fn solve_puzzle(solver_data: &SolverData) -> SolverResult {
    let mut piece_used = vec![false; 257];
    let mut cumulative_heuristic_side_count = vec![0u8; 256];
    let mut piece_index_to_try_next = vec![0u8; 256];
    let mut cumulative_breaks = vec![0u8; 256];
    let mut solve_index_counts: [u64; 257] = [0; 257];
    solve_index_counts[0] = 0; // Avoid warning when unused.
    let mut board = [RotatedPiece::default(); 256];

    let mut rng = rand::rng();

    let mut bottom_sides: Vec<Vec<RotatedPiece>> = vec![vec![]; 529];
    for (key, value) in &solver_data.bottom_side_pieces_rotated {
        let mut pieces: Vec<(RotatedPiece, i32)> = value
            .iter()
            .map(|x| {
                let score = if x.rotated_piece.heuristic_side_count > 0 {
                    100
                } else {
                    0
                } + rng.random_range(0..99);
                (x.rotated_piece, score)
            })
            .collect();
        pieces.sort_by(|a, b| b.1.cmp(&a.1));
        bottom_sides[*key as usize] = pieces.into_iter().map(|(p, _)| p).collect();
    }

    // Get first corner piece.
    let corner_list = &solver_data.corners[0];
    if !corner_list.is_empty() {
        let idx = rng.random_range(0..corner_list.len());
        board[0] = corner_list[idx];
    }

    piece_used[board[0].reid as usize] = true;
    cumulative_breaks[0] = 0;
    cumulative_heuristic_side_count[0] = board[0].heuristic_side_count;

    let mut solve_index: usize = 1;
    let mut max_solve_index = solve_index;
    let mut node_count: u64 = 0;

    loop {
        node_count += 1;

        // Uncomment to get this info printed.
        solve_index_counts[solve_index] += 1;

        if solve_index > max_solve_index {
            max_solve_index = solve_index;
            if solve_index >= MIN_SOLVE_INDEX_TO_SAVE {
                let board_to_save = board;
                util::save_board(&board_to_save, solve_index as u16);
                if solve_index >= 256 {
                    return SolverResult {
                        solve_indexes: solve_index_counts,
                        max_depth: max_solve_index,
                    };
                }
            }
        }

        if node_count > MAX_NODE_COUNT {
            return SolverResult {
                solve_indexes: solve_index_counts,
                max_depth: max_solve_index,
            };
        }

        let row = solver_data.board_search_sequence[solve_index].row as usize;
        let col = solver_data.board_search_sequence[solve_index].column as usize;
        let b_index = row * 16 + col;

        if board[b_index].reid > 0 {
            piece_used[board[b_index].reid as usize] = false;
            board[b_index].reid = 0;
        }

        let candidates: &Vec<RotatedPiece> = if row == 0 {
            let key = (board[row * 16 + (col - 1)].right as usize) * 23;
            if col < 15 {
                bottom_sides[key].as_ref()
            } else {
                solver_data.corners[key].as_ref()
            }
        } else {
            let left_side = if col == 0 {
                0
            } else {
                board[row * 16 + (col - 1)].right
            };
            let key = (left_side as usize) * 23 + (board[(row - 1) * 16 + col].top as usize);
            solver_data.get_pieces(solver_data.master_piece_lookup[b_index])[key].as_ref()
        };

        let mut found_piece = false;

        let breaks_this_turn =
            solver_data.break_array[solve_index] - cumulative_breaks[solve_index - 1];
        let try_index = piece_index_to_try_next[solve_index] as usize;
        let piece_candidate_length = candidates.len();

        for i in try_index..piece_candidate_length {
            if candidates[i].breaks > breaks_this_turn {
                break;
            }

            if !piece_used[candidates[i].reid as usize] {
                if solve_index <= MAX_HEURISTIC_INDEX
                    && ((cumulative_heuristic_side_count[solve_index - 1]
                        + candidates[i].heuristic_side_count)
                        < solver_data.heuristic_array[solve_index] as u8)
                {
                    break;
                }

                found_piece = true;
                let piece = candidates[i];
                board[b_index] = piece;
                piece_used[piece.reid as usize] = true;
                cumulative_breaks[solve_index] = cumulative_breaks[solve_index - 1] + piece.breaks;
                cumulative_heuristic_side_count[solve_index] =
                    cumulative_heuristic_side_count[solve_index - 1] + piece.heuristic_side_count;
                piece_index_to_try_next[solve_index] = (i + 1) as u8;
                solve_index += 1;
                break;
            }
        }

        if !found_piece {
            piece_index_to_try_next[solve_index] = 0;
            solve_index -= 1;
        }
    }
}
