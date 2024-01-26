use crate::pieces::PIECES;
use crate::structs::{
    Piece, RotatedPieceId, RotatedPieceWithLeftBottom, SearchIndex, SolverResult,
};
use crate::utils::{
    first_break_index, get_board_order, get_break_array, get_rotated_pieces, reset_caches,
    save_board, ROTATED_PIECES,
};
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use thousands::Separable;

mod board_order;
mod pieces;
mod structs;
mod utils;

const MAX_HEURISTIC_INDEX: usize = 160;

fn get_num_cores() -> usize {
    match env::var("CORES") {
        Ok(value) => value.parse::<usize>().unwrap(),
        Err(_e) => num_cpus::get(),
    }
}

fn main() {
    let num_virtual_cores = get_num_cores();
    println!("Using {num_virtual_cores} cores");
    let overall_stopwatch = Instant::now();

    let max_depth = Arc::new(Mutex::new(0));
    let mut total_index_count: u64 = 0;
    let mut loop_count: u64 = 0;

    let empty_vec: Vec<Vec<RotatedPieceId>> = vec![vec![]];

    unsafe {
        loop {
            loop_count += 1;

            reset_caches();
            let data = prepare_pieces_and_heuristics();
            let data2 = prepare_master_piece_lookup(&data, &empty_vec);
            println!("Solving with {num_virtual_cores} cores...");

            let index_counts: Arc<Mutex<HashMap<u32, u64>>> = Arc::new(Mutex::new(HashMap::new()));

            // This only num_virtual_cores-1 threads; we need to save one for the us.
            (1..num_virtual_cores).into_par_iter().for_each(|core| {
                let max_depth = Arc::clone(&max_depth);
                let index_counts = Arc::clone(&index_counts);

                for repeat in 1..=5 {
                    println!("Core {core}: start loop {loop_count}, repeat {repeat}");
                    let stopwatch = Instant::now();
                    let solver_result = solve_puzzle(&data, &data2);

                    {
                        let mut index_counts = index_counts.lock().unwrap();
                        for j in 0..=256 {
                            let count = solver_result.solve_indexes[j];
                            (*index_counts)
                                .entry(j as u32)
                                .and_modify(|e| *e += count)
                                .or_insert(count);
                        }
                    }

                    {
                        let mut max_depth = max_depth.lock().unwrap();
                        if solver_result.max_depth > *max_depth {
                            *max_depth = solver_result.max_depth;
                        }
                    }

                    println!(
                        "Core {core}: finish loop {loop_count}, repeat {repeat} in {} seconds",
                        stopwatch.elapsed().as_secs()
                    );
                }
            });
            println!("Result"); // No equivalent to C# Parallel.For result.

            // This will only print valid numbers if you let the solver count how far you are.
            let index_counts_clone = index_counts.clone();
            let index_counts_locked = index_counts_clone.lock().unwrap();
            for i in 0..=256 {
                if index_counts_locked[&i] != 0 {
                    println!("{i} {}", index_counts_locked[&i].separate_with_commas());
                }
                total_index_count += index_counts_locked[&i];
            }

            let elapsed_time_seconds = overall_stopwatch.elapsed().as_secs();
            let rate = total_index_count / elapsed_time_seconds;
            println!(
                "Total {} nodes in {elapsed_time_seconds} seconds, {} per second, max depth {}",
                total_index_count.separate_with_commas(),
                rate.separate_with_commas(),
                *max_depth.lock().unwrap()
            );
        }
    }
}

unsafe fn solve_puzzle(data: &Data, data2: &Data2) -> SolverResult {
    let mut piece_used: [bool; 257] = [false; 257];
    let mut cumulative_heuristic_side_count: [u8; 256] = [0; 256];
    let mut piece_index_to_try_next: [u8; 256] = [0; 256];
    let mut cumulative_breaks: [u8; 256] = [0; 256];
    let mut solve_index_counts: [u64; 257] = [0; 257];
    solve_index_counts[0] = 0; // Avoid warning when unused.
    let mut board: [RotatedPieceId; 256] = [0; 256];

    let mut rng = rand::thread_rng();

    let mut bottom_sides: Vec<Vec<RotatedPieceId>> = vec![vec![]; 529];

    for (key, value) in data.bottom_side_pieces_rotated.iter() {
        let mut sorted_pieces = value.clone();
        sorted_pieces.sort_by(|a, b| unsafe {
            let score_a = (if ROTATED_PIECES[a.rotated_piece_id].heuristic_side_count > 0 {
                100
            } else {
                0
            }) + rng.gen_range(0..99);
            let score_b = (if ROTATED_PIECES[b.rotated_piece_id].heuristic_side_count > 0 {
                100
            } else {
                0
            }) + rng.gen_range(0..99);
            score_b.cmp(&score_a) // Descending order.
        });

        bottom_sides[*key as usize] = sorted_pieces
            .into_iter()
            .map(|x| x.rotated_piece_id)
            .collect();
    }

    if let Some(first_corner) = data.corners.first() {
        if let Some(piece_id) = first_corner.iter().min_by_key(|_| rng.gen_range(1..1000)) {
            board[0] = *piece_id;
        }
    }

    piece_used[ROTATED_PIECES[board[0]].piece_number as usize] = true;
    cumulative_breaks[0] = 0;
    cumulative_heuristic_side_count[0] = ROTATED_PIECES[board[0]].heuristic_side_count;

    let mut solve_index: usize = 1; // This goes from 0....255; we've solved #0 already, so start at #1.
    let mut max_solve_index: usize = solve_index;
    let mut node_count: u64 = 0;

    loop {
        node_count += 1;

        // Uncomment to get this info printed.
        solve_index_counts[solve_index] += 1;

        if solve_index > max_solve_index {
            max_solve_index = solve_index;

            if solve_index >= 240 {
                save_board(&board, max_solve_index);

                if solve_index >= 256 {
                    return SolverResult {
                        solve_indexes: solve_index_counts,
                        max_depth: max_solve_index,
                    };
                }
            }
        }

        if node_count > 500_000_000 {
            return SolverResult {
                solve_indexes: solve_index_counts,
                max_depth: max_solve_index,
            };
        }

        let row = data.board_search_sequence[solve_index].row as usize;
        let col = data.board_search_sequence[solve_index].col as usize;

        if ROTATED_PIECES[board[row * 16 + col]].piece_number > 0 {
            piece_used[ROTATED_PIECES[board[row * 16 + col]].piece_number as usize] = false;
            board[row * 16 + col] = 0;
        }

        let piece_candidates: &Vec<RotatedPieceId> = if row != 0 {
            let left_side = if col == 0 {
                0
            } else {
                ROTATED_PIECES[board[row * 16 + (col - 1)]].right as usize
            };
            let x = data2.master_piece_lookup[row * 16 + col];
            &x[left_side * 23 + ROTATED_PIECES[board[(row - 1) * 16 + col]].top as usize]
        } else if col < 15 {
            &bottom_sides[ROTATED_PIECES[board[row * 16 + (col - 1)]].right as usize * 23]
        } else {
            &data.corners[ROTATED_PIECES[board[row * 16 + (col - 1)]].right as usize * 23]
        };

        let mut found_piece = false;

        if !piece_candidates.is_empty() {
            let breaks_this_turn =
                data.break_array[solve_index] - cumulative_breaks[solve_index - 1];
            let try_index = piece_index_to_try_next[solve_index] as usize;

            let piece_candidate_length = piece_candidates.len();
            for i in try_index..piece_candidate_length {
                if ROTATED_PIECES[piece_candidates[i]].break_count > breaks_this_turn {
                    break;
                }

                if !piece_used[ROTATED_PIECES[piece_candidates[i]].piece_number as usize] {
                    if solve_index <= MAX_HEURISTIC_INDEX
                        && u32::from(
                            cumulative_heuristic_side_count[solve_index - 1]
                                + ROTATED_PIECES[piece_candidates[i]].heuristic_side_count,
                        ) < data.heuristic_array[solve_index]
                    {
                        break;
                    }

                    found_piece = true;

                    let piece_id = piece_candidates[i];

                    board[row * 16 + col] = piece_id;
                    piece_used[ROTATED_PIECES[piece_id].piece_number as usize] = true;

                    cumulative_breaks[solve_index] =
                        cumulative_breaks[solve_index - 1] + ROTATED_PIECES[piece_id].break_count;
                    cumulative_heuristic_side_count[solve_index] = cumulative_heuristic_side_count
                        [solve_index - 1]
                        + ROTATED_PIECES[piece_id].heuristic_side_count;

                    piece_index_to_try_next[solve_index] = (i + 1) as u8;
                    solve_index += 1;
                    break;
                }
            }
        }

        if !found_piece {
            piece_index_to_try_next[solve_index] = 0;
            solve_index -= 1;
        }
    }
}

struct Data {
    corners: Vec<Vec<RotatedPieceId>>,
    left_sides: Vec<Vec<RotatedPieceId>>,
    right_sides_with_breaks: Vec<Vec<RotatedPieceId>>,
    right_sides_without_breaks: Vec<Vec<RotatedPieceId>>,
    top_sides: Vec<Vec<RotatedPieceId>>,
    middles_with_break: Vec<Vec<RotatedPieceId>>,
    middles_no_break: Vec<Vec<RotatedPieceId>>,
    south_start: Vec<Vec<RotatedPieceId>>,
    west_start: Vec<Vec<RotatedPieceId>>,
    start: Vec<Vec<RotatedPieceId>>,
    bottom_side_pieces_rotated: HashMap<u16, Vec<RotatedPieceWithLeftBottom>>,
    board_search_sequence: [SearchIndex; 256],
    break_array: [u8; 256],
    heuristic_array: [u32; 256],
}

struct Data2<'a> {
    master_piece_lookup: [&'a Vec<Vec<RotatedPieceId>>; 256],
}

unsafe fn prepare_pieces_and_heuristics() -> Data {
    let corner_pieces: Vec<&Piece> = PIECES
        .iter()
        .filter(|piece| piece.piece_type == 2)
        .collect();
    let side_pieces: Vec<&Piece> = PIECES
        .iter()
        .filter(|piece| piece.piece_type == 1)
        .collect();
    // Exclude start piece.
    let middle_pieces: Vec<&Piece> = PIECES
        .iter()
        .filter(|piece| piece.piece_type == 0 && piece.piece_number != 139)
        .collect();
    let start_piece: Vec<&Piece> = PIECES
        .iter()
        .filter(|piece| piece.piece_number == 139)
        .collect();

    // Corners
    let corner_pieces_rotated = build_rotated_array2(&corner_pieces, |_piece| true, false);

    // Sides
    let sides_without_breaks: Vec<RotatedPieceWithLeftBottom> = side_pieces
        .iter()
        .flat_map(|piece| get_rotated_pieces(piece, false))
        .collect();
    let sides_with_breaks: Vec<RotatedPieceWithLeftBottom> = side_pieces
        .iter()
        .flat_map(|piece| get_rotated_pieces(piece, true))
        .collect();

    let bottom_side_pieces_rotated = build_rotated_array(&sides_without_breaks, |piece| unsafe {
        ROTATED_PIECES[piece.rotated_piece_id].rotations == 0
    });
    let left_side_pieces_rotated = build_rotated_array(&sides_without_breaks, |piece| unsafe {
        ROTATED_PIECES[piece.rotated_piece_id].rotations == 1
    });
    let right_side_pieces_with_breaks_rotated =
        build_rotated_array(&sides_with_breaks, |piece| unsafe {
            ROTATED_PIECES[piece.rotated_piece_id].rotations == 3
        });
    let right_side_pieces_without_breaks_rotated =
        build_rotated_array(&sides_without_breaks, |piece| unsafe {
            ROTATED_PIECES[piece.rotated_piece_id].rotations == 3
        });
    let top_side_pieces_rotated = build_rotated_array(&sides_with_breaks, |piece| unsafe {
        ROTATED_PIECES[piece.rotated_piece_id].rotations == 2
    });

    // Middles
    let middle_pieces_rotated_with_breaks =
        build_rotated_array2(&middle_pieces, |_piece| true, true);
    let middle_pieces_rotated_without_breaks =
        build_rotated_array2(&middle_pieces, |_piece| true, false);
    let south_start_piece_rotated = build_rotated_array2(
        &middle_pieces,
        |piece| ROTATED_PIECES[piece.rotated_piece_id].top == 6,
        false,
    );
    let west_start_piece_rotated = build_rotated_array2(
        &middle_pieces,
        |piece| ROTATED_PIECES[piece.rotated_piece_id].right == 11,
        false,
    );
    let start_piece_rotated = build_rotated_array2(
        &start_piece,
        |piece| ROTATED_PIECES[piece.rotated_piece_id].rotations == 2,
        false,
    );

    let corners = build_array(&corner_pieces_rotated);
    let left_sides = build_array(&left_side_pieces_rotated);
    let top_sides = build_array(&top_side_pieces_rotated);
    let right_sides_with_breaks = build_array(&right_side_pieces_with_breaks_rotated);
    let right_sides_without_breaks = build_array(&right_side_pieces_without_breaks_rotated);
    let middles_with_break = build_array(&middle_pieces_rotated_with_breaks);
    let middles_no_break = build_array(&middle_pieces_rotated_without_breaks);
    let south_start = build_array(&south_start_piece_rotated);
    let west_start = build_array(&west_start_piece_rotated);
    let start = build_array(&start_piece_rotated);

    let board_search_sequence = get_board_order();
    let break_array = get_break_array();

    let mut heuristic_array: [u32; 256] = [0; 256];
    #[allow(clippy::needless_range_loop)]
    for i in 0..256 {
        heuristic_array[i] = if i <= 16 {
            0
        } else if i <= 26 {
            ((i as f64 - 16.0) * 2.8f64) as u32
        } else if i <= 56 {
            ((i as f64 - 26.0) * 1.43333f64 + 28.0) as u32
        } else if i <= 76 {
            ((i as f64 - 56.0) * 0.9f64 + 71.0) as u32
        } else if i <= 102 {
            ((i as f64 - 76.0) * 0.6538f64 + 89.0) as u32
        } else if i <= MAX_HEURISTIC_INDEX {
            ((i as f64 - 102.0) / 4.4615f64 + 106.0) as u32
        } else {
            0
        };
    }

    Data {
        corners,
        left_sides,
        right_sides_with_breaks,
        right_sides_without_breaks,
        top_sides,
        middles_with_break,
        middles_no_break,
        south_start,
        west_start,
        start,
        bottom_side_pieces_rotated,
        board_search_sequence,
        break_array,
        heuristic_array,
    }
}

fn prepare_master_piece_lookup<'a>(
    data: &'a Data,
    empty_vec: &'a Vec<Vec<RotatedPieceId>>,
) -> Data2<'a> {
    let mut master_piece_lookup: [&Vec<Vec<RotatedPieceId>>; 256] = [empty_vec; 256];

    for i in 0..256 {
        let row = data.board_search_sequence[i].row as usize;
        let col = data.board_search_sequence[i].col as usize;

        master_piece_lookup[row * 16 + col] = match row {
            15 => {
                if col == 15 || col == 0 {
                    &data.corners
                } else {
                    &data.top_sides
                }
            }
            0 => {
                // Don't populate the master lookup table since we randomize every time.
                empty_vec
            }
            _ => match col {
                15 => {
                    if i < first_break_index() {
                        &data.right_sides_without_breaks
                    } else {
                        &data.right_sides_with_breaks
                    }
                }
                0 => &data.left_sides,
                _ => match row {
                    7 => match col {
                        7 => &data.start,
                        6 => &data.west_start,
                        _ => {
                            if i < first_break_index() {
                                &data.middles_no_break
                            } else {
                                &data.middles_with_break
                            }
                        }
                    },
                    6 => {
                        if col == 7 {
                            &data.south_start
                        } else if i < first_break_index() {
                            &data.middles_no_break
                        } else {
                            &data.middles_with_break
                        }
                    }
                    _ => {
                        if i < first_break_index() {
                            &data.middles_no_break
                        } else {
                            &data.middles_with_break
                        }
                    }
                },
            },
        }
    }

    Data2 {
        master_piece_lookup,
    }
}

fn build_array(input: &HashMap<u16, Vec<RotatedPieceWithLeftBottom>>) -> Vec<Vec<RotatedPieceId>> {
    let mut rng = rand::thread_rng();
    let mut output: Vec<Vec<RotatedPieceId>> = vec![vec![]; 529];

    for (key, value) in input {
        let mut sorted_pieces = value.clone();
        sorted_pieces.sort_by(|a, b| {
            (b.score + rng.gen_range(0..99)).cmp(&(a.score + rng.gen_range(0..99)))
        });
        output[*key as usize] = sorted_pieces
            .into_iter()
            .map(|x| x.rotated_piece_id)
            .collect();
    }
    output
}

fn build_rotated_array(
    input: &[RotatedPieceWithLeftBottom],
    f: fn(&&RotatedPieceWithLeftBottom) -> bool,
) -> HashMap<u16, Vec<RotatedPieceWithLeftBottom>> {
    let mut groups: HashMap<u16, Vec<RotatedPieceWithLeftBottom>> = HashMap::new();

    for piece in input.iter().filter(f) {
        groups
            .entry(piece.left_bottom)
            .or_default()
            .push(piece.clone());
    }
    groups
}

fn build_rotated_array2(
    input: &[&Piece],
    f: fn(&RotatedPieceWithLeftBottom) -> bool,
    allow_breaks: bool,
) -> HashMap<u16, Vec<RotatedPieceWithLeftBottom>> {
    let mut groups: HashMap<u16, Vec<RotatedPieceWithLeftBottom>> = HashMap::new();

    for rotated_piece in input
        .iter()
        .flat_map(|piece| get_rotated_pieces(piece, allow_breaks))
        .filter(f)
    {
        groups
            .entry(rotated_piece.left_bottom)
            .or_default()
            .push(rotated_piece);
    }
    groups
}
