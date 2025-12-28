use crate::bits::{clear_bit, is_clear, set_bit};
use crate::config::{MAX_HEURISTIC_INDEX, MAX_NODE_COUNT, MIN_SOLVE_INDEX_TO_SAVE};
use crate::solver_data::SolverData;
use crate::structs::{RotatedPiece, SolverResult};
use crate::util::save_board;
use rand::Rng;

pub fn solve_puzzle(solver_data: &SolverData) -> SolverResult {
    let mut piece_used = [0u64; 5];
    let mut cumulative_heuristic_side_count = [0u8; 256];
    let mut piece_index_to_try_next = [0u8; 256];
    let mut cumulative_breaks = [0u8; 256];
    let mut solve_index_counts = [0u64; 257];
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

    let corner_list = &solver_data.corners[0];
    let first_corner_piece = corner_list[rng.random_range(0..solver_data.corners[0].len())];

    set_bit(&mut piece_used, first_corner_piece.reid as usize);
    cumulative_heuristic_side_count[0] = first_corner_piece.heuristic_side_count;
    cumulative_breaks[0] = 0;
    board[0] = first_corner_piece;

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
                save_board(&board, solve_index as u16);
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
            clear_bit(&mut piece_used, board[b_index].reid as usize);
            board[b_index].reid = 0;
        }

        let candidates: &Vec<RotatedPiece> = if row == 0 {
            let key = (board[col - 1].right as usize) * 23;
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

        #[allow(clippy::needless_range_loop)]
        for i in try_index..piece_candidate_length {
            if candidates[i].breaks > breaks_this_turn {
                break;
            }

            if is_clear(&piece_used, candidates[i].reid as usize) {
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
                set_bit(&mut piece_used, piece.reid as usize);
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
