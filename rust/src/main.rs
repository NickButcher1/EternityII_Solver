use crate::board_order::get_board_order;
use crate::config::{MAX_NODE_COUNT, MIN_SOLVE_INDEX_TO_SAVE};
use crate::structs::{RotatedPiece, RotatedPieceWithLeftBottom, SearchIndex};
use rand::Rng;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Instant;

mod board_order;
mod config;
mod pieces;
mod structs;
mod util;

const MAX_HEURISTIC_INDEX: usize = 160;

struct SolverData {
    corners: Vec<Option<Vec<RotatedPiece>>>,
    left_sides: Vec<Option<Vec<RotatedPiece>>>,
    right_sides_with_breaks: Vec<Option<Vec<RotatedPiece>>>,
    right_sides_without_breaks: Vec<Option<Vec<RotatedPiece>>>,
    top_sides: Vec<Option<Vec<RotatedPiece>>>,
    middles_with_break: Vec<Option<Vec<RotatedPiece>>>,
    middles_no_break: Vec<Option<Vec<RotatedPiece>>>,
    south_start: Vec<Option<Vec<RotatedPiece>>>,
    west_start: Vec<Option<Vec<RotatedPiece>>>,
    start: Vec<Option<Vec<RotatedPiece>>>,
    bottom_side_pieces_rotated: HashMap<u16, Vec<RotatedPieceWithLeftBottom>>,
    master_piece_lookup: Vec<Option<Vec<Option<Vec<RotatedPiece>>>>>,
    board_search_sequence: [SearchIndex; 256],
    break_array: [u8; 256],
    heuristic_array: Vec<i32>,
}

fn get_num_cores() -> usize {
    // Save one core to avoid grinding the system to a halt.
    match env::var("CORES") {
        Ok(value) => value.parse::<usize>().unwrap() - 1,
        Err(_e) => num_cpus::get() - 1,
    }
}

fn main() {
    let num_virtual_cores = get_num_cores();

    loop {
        // Solve for Eternity.
        let solver_data = Arc::new(prepare_pieces_and_heuristics());

        println!("Solving with {num_virtual_cores} cores...");

        let index_counts = Arc::new(Mutex::new(vec![0i64; 257]));

        // Create thread handles
        let mut handles = vec![];

        for _ in 0..num_virtual_cores {
            let index_counts_clone = Arc::clone(&index_counts);
            let solver_data_clone = Arc::clone(&solver_data);

            let handle = std::thread::spawn(move || {
                for _x in 0..5 {
                    let stopwatch = Instant::now();
                    let solve_indexes = solve_puzzle(&solver_data_clone);

                    let mut counts = index_counts_clone.lock().unwrap();
                    for j in 0..257 {
                        counts[j] += solve_indexes[j];
                    }
                    drop(counts);

                    let _elapsed = stopwatch.elapsed();
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let final_counts = index_counts.lock().unwrap();
        for i in 0..257 {
            println!("{} {}", i, final_counts[i]);
        }
    }
}

fn solve_puzzle(solver_data: &SolverData) -> Vec<i64> {
    let mut piece_used = vec![false; 257];
    let mut cumulative_heuristic_side_count = vec![0u8; 256];
    let mut piece_index_to_try_next = vec![0u8; 256];
    let mut cumulative_breaks = vec![0u8; 256];
    let solve_index_counts = vec![0i64; 257];
    let mut board = [RotatedPiece::default(); 256];

    let mut rng = rand::rng();

    let mut bottom_sides: Vec<Option<Vec<RotatedPiece>>> = vec![None; 529];
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
        bottom_sides[*key as usize] = Some(pieces.into_iter().map(|(p, _)| p).collect());
    }

    // Get first corner piece
    if let Some(ref corner_list) = solver_data.corners[0] {
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

        if solve_index > max_solve_index {
            max_solve_index = solve_index;
            if solve_index >= MIN_SOLVE_INDEX_TO_SAVE {
                let board_to_save = board;
                util::save_board(&board_to_save, solve_index as u16);
                if solve_index >= 256 {
                    return solve_index_counts;
                }
            }
        }

        if node_count > MAX_NODE_COUNT {
            return solve_index_counts;
        }

        let row = solver_data.board_search_sequence[solve_index].row as usize;
        let col = solver_data.board_search_sequence[solve_index].column as usize;

        if board[row * 16 + col].reid > 0 {
            piece_used[board[row * 16 + col].reid as usize] = false;
            board[row * 16 + col].reid = 0;
        }

        let piece_candidates: Option<&Vec<RotatedPiece>> = if row == 0 {
            if col < 15 {
                let key = (board[row * 16 + (col - 1)].right as usize) * 23;
                bottom_sides[key].as_ref()
            } else {
                let key = (board[row * 16 + (col - 1)].right as usize) * 23;
                solver_data.corners[key].as_ref()
            }
        } else {
            let left_side = if col == 0 {
                0
            } else {
                board[row * 16 + (col - 1)].right
            };
            let key = (left_side as usize) * 23 + (board[(row - 1) * 16 + col].top as usize);

            if let Some(ref lookup) = solver_data.master_piece_lookup[row * 16 + col] {
                lookup[key].as_ref()
            } else {
                None
            }
        };

        let mut found_piece = false;

        if let Some(candidates) = piece_candidates {
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
                    board[row * 16 + col] = piece;
                    piece_used[piece.reid as usize] = true;
                    cumulative_breaks[solve_index] =
                        cumulative_breaks[solve_index - 1] + piece.breaks;
                    cumulative_heuristic_side_count[solve_index] = cumulative_heuristic_side_count
                        [solve_index - 1]
                        + piece.heuristic_side_count;
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

fn prepare_pieces_and_heuristics() -> SolverData {
    let board_pieces = pieces::PIECES;

    let corner_pieces: Vec<_> = board_pieces
        .iter()
        .filter(|x| x.piece_type() == 2)
        .cloned()
        .collect();

    let side_pieces: Vec<_> = board_pieces
        .iter()
        .filter(|x| x.piece_type() == 1)
        .cloned()
        .collect();

    let middle_pieces: Vec<_> = board_pieces
        .iter()
        .filter(|x| x.piece_type() == 0 && x.reid != 139)
        .cloned()
        .collect();

    let start_piece: Vec<_> = board_pieces
        .iter()
        .filter(|x| x.reid == 139)
        .cloned()
        .collect();

    // Corners
    let corner_pieces_rotated = group_by_left_bottom(
        corner_pieces
            .iter()
            .flat_map(|x| util::get_rotated_pieces(x, false))
            .collect(),
    );

    // Sides
    let sides_without_breaks: Vec<_> = side_pieces
        .iter()
        .flat_map(|x| util::get_rotated_pieces(x, false))
        .collect();

    let sides_with_breaks: Vec<_> = side_pieces
        .iter()
        .flat_map(|x| util::get_rotated_pieces(x, true))
        .collect();

    let bottom_side_pieces_rotated = group_by_left_bottom(
        sides_without_breaks
            .iter()
            .filter(|x| x.rotated_piece.rotations == 0)
            .cloned()
            .collect(),
    );

    let left_side_pieces_rotated = group_by_left_bottom(
        sides_without_breaks
            .iter()
            .filter(|x| x.rotated_piece.rotations == 1)
            .cloned()
            .collect(),
    );

    let right_side_pieces_with_breaks_rotated = group_by_left_bottom(
        sides_with_breaks
            .iter()
            .filter(|x| x.rotated_piece.rotations == 3)
            .cloned()
            .collect(),
    );

    let right_side_pieces_without_breaks_rotated = group_by_left_bottom(
        sides_without_breaks
            .iter()
            .filter(|x| x.rotated_piece.rotations == 3)
            .cloned()
            .collect(),
    );

    let top_side_pieces_rotated = group_by_left_bottom(
        sides_with_breaks
            .iter()
            .filter(|x| x.rotated_piece.rotations == 2)
            .cloned()
            .collect(),
    );

    // Middles
    let middle_pieces_rotated_with_breaks = group_by_left_bottom(
        middle_pieces
            .iter()
            .flat_map(|x| util::get_rotated_pieces(x, true))
            .collect(),
    );

    let middle_pieces_rotated_without_breaks = group_by_left_bottom(
        middle_pieces
            .iter()
            .flat_map(|x| util::get_rotated_pieces(x, false))
            .collect(),
    );

    let south_start_piece_rotated = group_by_left_bottom(
        middle_pieces
            .iter()
            .flat_map(|x| util::get_rotated_pieces(x, false))
            .filter(|x| x.rotated_piece.top == 6)
            .collect(),
    );

    let west_start_piece_rotated = group_by_left_bottom(
        middle_pieces
            .iter()
            .flat_map(|x| util::get_rotated_pieces(x, false))
            .filter(|x| x.rotated_piece.right == 11)
            .collect(),
    );

    let start_piece_rotated = group_by_left_bottom(
        start_piece
            .iter()
            .flat_map(|x| util::get_rotated_pieces(x, false))
            .filter(|x| x.rotated_piece.rotations == 2)
            .collect(),
    );

    let mut rng = rand::rng();

    let corners = create_sorted_array(&corner_pieces_rotated, &mut rng);
    let left_sides = create_sorted_array(&left_side_pieces_rotated, &mut rng);
    let top_sides = create_sorted_array(&top_side_pieces_rotated, &mut rng);
    let right_sides_with_breaks =
        create_sorted_array(&right_side_pieces_with_breaks_rotated, &mut rng);
    let right_sides_without_breaks =
        create_sorted_array(&right_side_pieces_without_breaks_rotated, &mut rng);
    let middles_with_break = create_sorted_array(&middle_pieces_rotated_with_breaks, &mut rng);
    let middles_no_break = create_sorted_array(&middle_pieces_rotated_without_breaks, &mut rng);
    let south_start = create_sorted_array(&south_start_piece_rotated, &mut rng);
    let west_start = create_sorted_array(&west_start_piece_rotated, &mut rng);
    let start = create_sorted_array(&start_piece_rotated, &mut rng);

    let board_search_sequence = get_board_order();
    let break_array = util::get_break_array();

    let mut master_piece_lookup: Vec<Option<Vec<Option<Vec<RotatedPiece>>>>> = vec![None; 256];

    #[allow(clippy::needless_range_loop)]
    for i in 0..256 {
        let row = board_search_sequence[i].row as usize;
        let col = board_search_sequence[i].column as usize;

        let lookup = if row == 15 {
            if col == 15 || col == 0 {
                Some(corners.clone())
            } else {
                Some(top_sides.clone())
            }
        } else if row == 0 {
            None
        } else if col == 15 {
            if i < util::first_break_index() {
                Some(right_sides_without_breaks.clone())
            } else {
                Some(right_sides_with_breaks.clone())
            }
        } else if col == 0 {
            Some(left_sides.clone())
        } else if row == 7 {
            if col == 7 {
                Some(start.clone())
            } else if col == 6 {
                Some(west_start.clone())
            } else if i < util::first_break_index() {
                Some(middles_no_break.clone())
            } else {
                Some(middles_with_break.clone())
            }
        } else if row == 6 {
            if col == 7 {
                Some(south_start.clone())
            } else if i < util::first_break_index() {
                Some(middles_no_break.clone())
            } else {
                Some(middles_with_break.clone())
            }
        } else if i < util::first_break_index() {
            Some(middles_no_break.clone())
        } else {
            Some(middles_with_break.clone())
        };

        master_piece_lookup[row * 16 + col] = lookup;
    }

    let mut heuristic_array = vec![0i32; 256];
    #[allow(clippy::needless_range_loop)]
    for i in 0..256 {
        heuristic_array[i] = if i <= 16 {
            0
        } else if i <= 26 {
            ((i as f32 - 16.0) * 2.8) as i32
        } else if i <= 56 {
            (((i as f32 - 26.0) * 1.43333) + 28.0) as i32
        } else if i <= 76 {
            (((i as f32 - 56.0) * 0.9) + 71.0) as i32
        } else if i <= 102 {
            (((i as f32 - 76.0) * 0.6538) + 89.0) as i32
        } else if i <= MAX_HEURISTIC_INDEX {
            (((i as f32 - 102.0) / 4.4615) + 106.0) as i32
        } else {
            0
        };
    }

    SolverData {
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
        master_piece_lookup,
        board_search_sequence,
        break_array,
        heuristic_array,
    }
}

fn group_by_left_bottom(
    pieces: Vec<RotatedPieceWithLeftBottom>,
) -> HashMap<u16, Vec<RotatedPieceWithLeftBottom>> {
    let mut map: HashMap<u16, Vec<RotatedPieceWithLeftBottom>> = HashMap::new();
    for piece in pieces {
        map.entry(piece.left_bottom).or_default().push(piece);
    }
    map
}

fn create_sorted_array(
    map: &HashMap<u16, Vec<RotatedPieceWithLeftBottom>>,
    rng: &mut impl Rng,
) -> Vec<Option<Vec<RotatedPiece>>> {
    let mut result = vec![None; 529];
    for (key, value) in map {
        let mut pieces: Vec<(RotatedPiece, i32)> = value
            .iter()
            .map(|x| (x.rotated_piece, x.score + rng.random_range(0..99)))
            .collect();
        pieces.sort_by(|a, b| b.1.cmp(&a.1));
        result[*key as usize] = Some(pieces.into_iter().map(|(p, _)| p).collect());
    }
    result
}

impl Clone for SolverData {
    fn clone(&self) -> Self {
        SolverData {
            corners: self.corners.clone(),
            left_sides: self.left_sides.clone(),
            right_sides_with_breaks: self.right_sides_with_breaks.clone(),
            right_sides_without_breaks: self.right_sides_without_breaks.clone(),
            top_sides: self.top_sides.clone(),
            middles_with_break: self.middles_with_break.clone(),
            middles_no_break: self.middles_no_break.clone(),
            south_start: self.south_start.clone(),
            west_start: self.west_start.clone(),
            start: self.start.clone(),
            bottom_side_pieces_rotated: self.bottom_side_pieces_rotated.clone(),
            master_piece_lookup: self.master_piece_lookup.clone(),
            board_search_sequence: self.board_search_sequence,
            break_array: self.break_array,
            heuristic_array: self.heuristic_array.clone(),
        }
    }
}
