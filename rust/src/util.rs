use crate::config::{BREAK_INDEXES_ALLOWED, HEURISTIC_SIDES};
use crate::pieces;
use crate::structs::{Piece, RotatedPiece, RotatedPieceWithLeftBottom};
use std::fs;

pub const SIDE_EDGES: &[u8] = &[1, 5, 9, 13, 17];

pub fn calculate_two_sides(side1: u16, side2: u16) -> u16 {
    (side1 * 23) + side2
}

pub fn get_rotated_pieces(piece: &Piece, allow_breaks: bool) -> Vec<RotatedPieceWithLeftBottom> {
    let mut score_base: i32 = 0;
    let mut heuristic_side_count: u8 = 0;

    // Calculate heuristic score
    for &side in HEURISTIC_SIDES {
        if piece.left_side == side {
            score_base += 100;
            heuristic_side_count += 1;
        }
        if piece.top_side == side {
            score_base += 100;
            heuristic_side_count += 1;
        }
        if piece.right_side == side {
            score_base += 100;
            heuristic_side_count += 1;
        }
        if piece.bottom_side == side {
            score_base += 100;
            heuristic_side_count += 1;
        }
    }

    let mut rotated_pieces = Vec::new();

    for left in 0..=22u16 {
        for bottom in 0..=22u16 {
            // Check all 4 rotations (0, 1, 2, 3)
            // Logic maps C# blocks to Rust. For brevity, I'll show the pattern
            // which can be repeated or refactored into a loop.

            // Rotation 0
            check_and_add_rotation(
                &mut rotated_pieces,
                piece,
                left,
                bottom,
                0,
                piece.left_side,
                piece.bottom_side,
                piece.top_side,
                piece.right_side,
                score_base,
                heuristic_side_count,
                allow_breaks,
            );

            // Rotation 1
            check_and_add_rotation(
                &mut rotated_pieces,
                piece,
                left,
                bottom,
                1,
                piece.bottom_side,
                piece.right_side,
                piece.left_side,
                piece.top_side,
                score_base,
                heuristic_side_count,
                allow_breaks,
            );

            // Rotation 2
            check_and_add_rotation(
                &mut rotated_pieces,
                piece,
                left,
                bottom,
                2,
                piece.right_side,
                piece.top_side,
                piece.bottom_side,
                piece.left_side,
                score_base,
                heuristic_side_count,
                allow_breaks,
            );

            // Rotation 3
            check_and_add_rotation(
                &mut rotated_pieces,
                piece,
                left,
                bottom,
                3,
                piece.top_side,
                piece.left_side,
                piece.right_side,
                piece.bottom_side,
                score_base,
                heuristic_side_count,
                allow_breaks,
            );
        }
    }
    rotated_pieces
}

/// Helper to reduce code duplication in the rotation loops
#[allow(clippy::too_many_arguments)]
fn check_and_add_rotation(
    list: &mut Vec<RotatedPieceWithLeftBottom>,
    piece: &Piece,
    target_l: u16,
    target_b: u16,
    rot_idx: u8,
    p_side_l: u8,
    p_side_b: u8,
    out_top: u8,
    out_right: u8,
    score_base: i32,
    h_count: u8,
    allow_breaks: bool,
) {
    let mut breaks: u8 = 0;
    let mut side_breaks: u8 = 0;

    if p_side_l != target_l as u8 {
        breaks += 1;
        if SIDE_EDGES.contains(&p_side_l) {
            side_breaks += 1;
        }
    }
    if p_side_b != target_b as u8 {
        breaks += 1;
        if SIDE_EDGES.contains(&p_side_b) {
            side_breaks += 1;
        }
    }

    if ((breaks == 0) || (breaks == 1 && allow_breaks)) && side_breaks == 0 {
        list.push(RotatedPieceWithLeftBottom {
            left_bottom: calculate_two_sides(target_l, target_b),
            score: score_base - (100_000 * breaks as i32),
            rotated_piece: RotatedPiece {
                piece_number: piece.piece_number,
                rotations: rot_idx,
                top_side: out_top,
                right_side: out_right,
                break_count: breaks,
                heuristic_side_count: h_count,
            },
        });
    }
}

pub fn save_board(board: &[RotatedPiece; 256], max_solve_index: u16) {
    let board_pieces = pieces::PIECES;
    let mut entire_board = String::new();
    let mut url_path = String::new();

    for i in (0..16).rev() {
        let mut row_str = String::new();
        for j in 0..16 {
            let p_rotated = board[i * 16 + j];
            if p_rotated.piece_number > 0 {
                row_str.push_str(&format!(
                    "{:>3}/{} ",
                    p_rotated.piece_number, p_rotated.rotations
                ));

                // Find original piece to get sides for URL
                if let Some(p) = board_pieces
                    .iter()
                    .find(|k| k.piece_number == p_rotated.piece_number)
                {
                    let (t, r, b, l) = match p_rotated.rotations {
                        0 => (p.top_side, p.right_side, p.bottom_side, p.left_side),
                        1 => (p.left_side, p.top_side, p.right_side, p.bottom_side),
                        2 => (p.bottom_side, p.left_side, p.top_side, p.right_side),
                        _ => (p.right_side, p.bottom_side, p.left_side, p.top_side),
                    };
                    for side in [t, r, b, l] {
                        url_path.push((side + b'a') as char);
                    }
                }
            } else {
                row_str.push_str("---/- ");
                url_path.push_str("aaaa");
            }
        }
        entire_board.push_str(&row_str);
        entire_board.push('\n');
    }

    let final_output = format!(
        "{entire_board}\nhttps://e2.bucas.name/#puzzle=Joshua_Blackwood&board_w=16&board_h=16&board_edges={url_path}&motifs_order=jblackwood",
    );

    let hash = format!("{:x}", md5::compute(&final_output));
    let filename = format!(
        "{}_{}_{}.txt",
        max_solve_index,
        hash,
        rand::random::<u32>() % 1_000_000
    );

    if let Some(mut path) = dirs::home_dir() {
        path.push("EternitySolutions");
        let _ = fs::create_dir_all(&path);
        path.push(filename);
        let _ = fs::write(path, final_output);
    }
}

pub fn first_break_index() -> usize {
    *BREAK_INDEXES_ALLOWED.iter().min().unwrap_or(&256)
}

pub fn get_break_array() -> [u8; 256] {
    let mut cumulative_breaks = [0u8; 256];
    let mut count = 0;
    #[allow(clippy::needless_range_loop)]
    for i in 0..256 {
        if BREAK_INDEXES_ALLOWED.contains(&i) {
            count += 1;
        }
        cumulative_breaks[i] = count;
    }
    cumulative_breaks
}
