use crate::structs::{Piece, RotatedPiece, RotatedPieceWithLeftBottom, SearchIndex};
use std::fs;

// Constants (replacing static readonly Lists)
pub const SIDE_EDGES: &[u8] = &[1, 5, 9, 13, 17];
// pub const MIDDLE_EDGES: &[u8] = &[2, 3, 4, 6, 7, 8, 10, 11, 12, 14, 15, 16, 18, 19, 20, 21, 22];
pub const HEURISTIC_SIDES: &[u8] = &[13, 16, 10];
const BREAK_INDEXES_ALLOWED: &[usize] = &[201, 206, 211, 216, 221, 225, 229, 233, 237, 239];

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
    let board_pieces = get_pieces();
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

    if let Some(mut path) = dirs::document_dir() {
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

pub fn get_board_order() -> [SearchIndex; 256] {
    let board_order_raw: [[i32; 16]; 16] = [
        [
            196, 197, 198, 199, 200, 205, 210, 215, 220, 225, 230, 235, 243, 249, 254, 255,
        ],
        [
            191, 192, 193, 194, 195, 204, 209, 214, 219, 224, 229, 234, 242, 248, 252, 253,
        ],
        [
            186, 187, 188, 189, 190, 203, 208, 213, 218, 223, 228, 233, 241, 247, 250, 251,
        ],
        [
            181, 182, 183, 184, 185, 202, 207, 212, 217, 222, 227, 232, 240, 244, 245, 246,
        ],
        [
            176, 177, 178, 179, 180, 201, 206, 211, 216, 221, 226, 231, 236, 237, 238, 239,
        ],
        [
            160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175,
        ],
        [
            144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159,
        ],
        [
            128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143,
        ],
        [
            112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127,
        ],
        [
            96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111,
        ],
        [
            80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
        ],
        [
            64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79,
        ],
        [
            48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
        ],
        [
            32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        ],
        [
            16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
        ],
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    ];

    let mut board_search_sequence = [SearchIndex { row: 0, column: 0 }; 256];
    for row in 0..16 {
        for col in 0..16 {
            let piece_seq_num = board_order_raw[15 - row][col] as usize;
            board_search_sequence[piece_seq_num] = SearchIndex {
                row: row as u8,
                column: col as u8,
            };
        }
    }
    board_search_sequence
}

pub fn get_pieces() -> Vec<Piece> {
    vec![
        Piece {
            piece_number: 1,
            top_side: 1,
            right_side: 17,
            bottom_side: 0,
            left_side: 0,
        },
        Piece {
            piece_number: 2,
            top_side: 1,
            right_side: 5,
            bottom_side: 0,
            left_side: 0,
        },
        Piece {
            piece_number: 3,
            top_side: 9,
            right_side: 17,
            bottom_side: 0,
            left_side: 0,
        },
        Piece {
            piece_number: 4,
            top_side: 17,
            right_side: 9,
            bottom_side: 0,
            left_side: 0,
        },
        Piece {
            piece_number: 5,
            top_side: 2,
            right_side: 1,
            bottom_side: 0,
            left_side: 1,
        },
        Piece {
            piece_number: 6,
            top_side: 10,
            right_side: 9,
            bottom_side: 0,
            left_side: 1,
        },
        Piece {
            piece_number: 7,
            top_side: 6,
            right_side: 1,
            bottom_side: 0,
            left_side: 1,
        },
        Piece {
            piece_number: 8,
            top_side: 6,
            right_side: 13,
            bottom_side: 0,
            left_side: 1,
        },
        Piece {
            piece_number: 9,
            top_side: 11,
            right_side: 17,
            bottom_side: 0,
            left_side: 1,
        },
        Piece {
            piece_number: 10,
            top_side: 7,
            right_side: 5,
            bottom_side: 0,
            left_side: 1,
        },
        Piece {
            piece_number: 11,
            top_side: 15,
            right_side: 9,
            bottom_side: 0,
            left_side: 1,
        },
        Piece {
            piece_number: 12,
            top_side: 8,
            right_side: 5,
            bottom_side: 0,
            left_side: 1,
        },
        Piece {
            piece_number: 13,
            top_side: 8,
            right_side: 13,
            bottom_side: 0,
            left_side: 1,
        },
        Piece {
            piece_number: 14,
            top_side: 21,
            right_side: 5,
            bottom_side: 0,
            left_side: 1,
        },
        Piece {
            piece_number: 15,
            top_side: 10,
            right_side: 1,
            bottom_side: 0,
            left_side: 9,
        },
        Piece {
            piece_number: 16,
            top_side: 18,
            right_side: 17,
            bottom_side: 0,
            left_side: 9,
        },
        Piece {
            piece_number: 17,
            top_side: 14,
            right_side: 13,
            bottom_side: 0,
            left_side: 9,
        },
        Piece {
            piece_number: 18,
            top_side: 19,
            right_side: 13,
            bottom_side: 0,
            left_side: 9,
        },
        Piece {
            piece_number: 19,
            top_side: 7,
            right_side: 9,
            bottom_side: 0,
            left_side: 9,
        },
        Piece {
            piece_number: 20,
            top_side: 15,
            right_side: 9,
            bottom_side: 0,
            left_side: 9,
        },
        Piece {
            piece_number: 21,
            top_side: 4,
            right_side: 5,
            bottom_side: 0,
            left_side: 9,
        },
        Piece {
            piece_number: 22,
            top_side: 12,
            right_side: 1,
            bottom_side: 0,
            left_side: 9,
        },
        Piece {
            piece_number: 23,
            top_side: 12,
            right_side: 13,
            bottom_side: 0,
            left_side: 9,
        },
        Piece {
            piece_number: 24,
            top_side: 20,
            right_side: 1,
            bottom_side: 0,
            left_side: 9,
        },
        Piece {
            piece_number: 25,
            top_side: 21,
            right_side: 1,
            bottom_side: 0,
            left_side: 9,
        },
        Piece {
            piece_number: 26,
            top_side: 2,
            right_side: 9,
            bottom_side: 0,
            left_side: 17,
        },
        Piece {
            piece_number: 27,
            top_side: 2,
            right_side: 17,
            bottom_side: 0,
            left_side: 17,
        },
        Piece {
            piece_number: 28,
            top_side: 10,
            right_side: 17,
            bottom_side: 0,
            left_side: 17,
        },
        Piece {
            piece_number: 29,
            top_side: 18,
            right_side: 17,
            bottom_side: 0,
            left_side: 17,
        },
        Piece {
            piece_number: 30,
            top_side: 7,
            right_side: 13,
            bottom_side: 0,
            left_side: 17,
        },
        Piece {
            piece_number: 31,
            top_side: 15,
            right_side: 9,
            bottom_side: 0,
            left_side: 17,
        },
        Piece {
            piece_number: 32,
            top_side: 20,
            right_side: 17,
            bottom_side: 0,
            left_side: 17,
        },
        Piece {
            piece_number: 33,
            top_side: 8,
            right_side: 9,
            bottom_side: 0,
            left_side: 17,
        },
        Piece {
            piece_number: 34,
            top_side: 8,
            right_side: 5,
            bottom_side: 0,
            left_side: 17,
        },
        Piece {
            piece_number: 35,
            top_side: 16,
            right_side: 13,
            bottom_side: 0,
            left_side: 17,
        },
        Piece {
            piece_number: 36,
            top_side: 22,
            right_side: 5,
            bottom_side: 0,
            left_side: 17,
        },
        Piece {
            piece_number: 37,
            top_side: 18,
            right_side: 1,
            bottom_side: 0,
            left_side: 5,
        },
        Piece {
            piece_number: 38,
            top_side: 3,
            right_side: 13,
            bottom_side: 0,
            left_side: 5,
        },
        Piece {
            piece_number: 39,
            top_side: 11,
            right_side: 13,
            bottom_side: 0,
            left_side: 5,
        },
        Piece {
            piece_number: 40,
            top_side: 19,
            right_side: 9,
            bottom_side: 0,
            left_side: 5,
        },
        Piece {
            piece_number: 41,
            top_side: 19,
            right_side: 17,
            bottom_side: 0,
            left_side: 5,
        },
        Piece {
            piece_number: 42,
            top_side: 15,
            right_side: 1,
            bottom_side: 0,
            left_side: 5,
        },
        Piece {
            piece_number: 43,
            top_side: 15,
            right_side: 9,
            bottom_side: 0,
            left_side: 5,
        },
        Piece {
            piece_number: 44,
            top_side: 15,
            right_side: 17,
            bottom_side: 0,
            left_side: 5,
        },
        Piece {
            piece_number: 45,
            top_side: 4,
            right_side: 1,
            bottom_side: 0,
            left_side: 5,
        },
        Piece {
            piece_number: 46,
            top_side: 20,
            right_side: 5,
            bottom_side: 0,
            left_side: 5,
        },
        Piece {
            piece_number: 47,
            top_side: 8,
            right_side: 5,
            bottom_side: 0,
            left_side: 5,
        },
        Piece {
            piece_number: 48,
            top_side: 16,
            right_side: 5,
            bottom_side: 0,
            left_side: 5,
        },
        Piece {
            piece_number: 49,
            top_side: 2,
            right_side: 13,
            bottom_side: 0,
            left_side: 13,
        },
        Piece {
            piece_number: 50,
            top_side: 10,
            right_side: 1,
            bottom_side: 0,
            left_side: 13,
        },
        Piece {
            piece_number: 51,
            top_side: 10,
            right_side: 9,
            bottom_side: 0,
            left_side: 13,
        },
        Piece {
            piece_number: 52,
            top_side: 6,
            right_side: 1,
            bottom_side: 0,
            left_side: 13,
        },
        Piece {
            piece_number: 53,
            top_side: 7,
            right_side: 5,
            bottom_side: 0,
            left_side: 13,
        },
        Piece {
            piece_number: 54,
            top_side: 4,
            right_side: 5,
            bottom_side: 0,
            left_side: 13,
        },
        Piece {
            piece_number: 55,
            top_side: 4,
            right_side: 13,
            bottom_side: 0,
            left_side: 13,
        },
        Piece {
            piece_number: 56,
            top_side: 8,
            right_side: 17,
            bottom_side: 0,
            left_side: 13,
        },
        Piece {
            piece_number: 57,
            top_side: 16,
            right_side: 1,
            bottom_side: 0,
            left_side: 13,
        },
        Piece {
            piece_number: 58,
            top_side: 16,
            right_side: 13,
            bottom_side: 0,
            left_side: 13,
        },
        Piece {
            piece_number: 59,
            top_side: 21,
            right_side: 9,
            bottom_side: 0,
            left_side: 13,
        },
        Piece {
            piece_number: 60,
            top_side: 22,
            right_side: 17,
            bottom_side: 0,
            left_side: 13,
        },
        Piece {
            piece_number: 61,
            top_side: 6,
            right_side: 18,
            bottom_side: 2,
            left_side: 2,
        },
        Piece {
            piece_number: 62,
            top_side: 14,
            right_side: 7,
            bottom_side: 2,
            left_side: 2,
        },
        Piece {
            piece_number: 63,
            top_side: 10,
            right_side: 3,
            bottom_side: 2,
            left_side: 10,
        },
        Piece {
            piece_number: 64,
            top_side: 2,
            right_side: 8,
            bottom_side: 2,
            left_side: 18,
        },
        Piece {
            piece_number: 65,
            top_side: 18,
            right_side: 22,
            bottom_side: 2,
            left_side: 18,
        },
        Piece {
            piece_number: 66,
            top_side: 14,
            right_side: 14,
            bottom_side: 2,
            left_side: 18,
        },
        Piece {
            piece_number: 67,
            top_side: 11,
            right_side: 10,
            bottom_side: 2,
            left_side: 18,
        },
        Piece {
            piece_number: 68,
            top_side: 20,
            right_side: 6,
            bottom_side: 2,
            left_side: 18,
        },
        Piece {
            piece_number: 69,
            top_side: 22,
            right_side: 8,
            bottom_side: 2,
            left_side: 18,
        },
        Piece {
            piece_number: 70,
            top_side: 3,
            right_side: 7,
            bottom_side: 2,
            left_side: 3,
        },
        Piece {
            piece_number: 71,
            top_side: 7,
            right_side: 12,
            bottom_side: 2,
            left_side: 3,
        },
        Piece {
            piece_number: 72,
            top_side: 14,
            right_side: 18,
            bottom_side: 2,
            left_side: 11,
        },
        Piece {
            piece_number: 73,
            top_side: 15,
            right_side: 4,
            bottom_side: 2,
            left_side: 11,
        },
        Piece {
            piece_number: 74,
            top_side: 20,
            right_side: 15,
            bottom_side: 2,
            left_side: 11,
        },
        Piece {
            piece_number: 75,
            top_side: 8,
            right_side: 3,
            bottom_side: 2,
            left_side: 11,
        },
        Piece {
            piece_number: 76,
            top_side: 14,
            right_side: 15,
            bottom_side: 2,
            left_side: 19,
        },
        Piece {
            piece_number: 77,
            top_side: 19,
            right_side: 15,
            bottom_side: 2,
            left_side: 19,
        },
        Piece {
            piece_number: 78,
            top_side: 3,
            right_side: 16,
            bottom_side: 2,
            left_side: 7,
        },
        Piece {
            piece_number: 79,
            top_side: 20,
            right_side: 3,
            bottom_side: 2,
            left_side: 7,
        },
        Piece {
            piece_number: 80,
            top_side: 16,
            right_side: 21,
            bottom_side: 2,
            left_side: 7,
        },
        Piece {
            piece_number: 81,
            top_side: 19,
            right_side: 18,
            bottom_side: 2,
            left_side: 15,
        },
        Piece {
            piece_number: 82,
            top_side: 18,
            right_side: 18,
            bottom_side: 2,
            left_side: 4,
        },
        Piece {
            piece_number: 83,
            top_side: 11,
            right_side: 4,
            bottom_side: 2,
            left_side: 4,
        },
        Piece {
            piece_number: 84,
            top_side: 18,
            right_side: 19,
            bottom_side: 2,
            left_side: 12,
        },
        Piece {
            piece_number: 85,
            top_side: 6,
            right_side: 14,
            bottom_side: 2,
            left_side: 12,
        },
        Piece {
            piece_number: 86,
            top_side: 8,
            right_side: 12,
            bottom_side: 2,
            left_side: 12,
        },
        Piece {
            piece_number: 87,
            top_side: 16,
            right_side: 20,
            bottom_side: 2,
            left_side: 12,
        },
        Piece {
            piece_number: 88,
            top_side: 2,
            right_side: 21,
            bottom_side: 2,
            left_side: 20,
        },
        Piece {
            piece_number: 89,
            top_side: 6,
            right_side: 22,
            bottom_side: 2,
            left_side: 20,
        },
        Piece {
            piece_number: 90,
            top_side: 4,
            right_side: 16,
            bottom_side: 2,
            left_side: 20,
        },
        Piece {
            piece_number: 91,
            top_side: 11,
            right_side: 12,
            bottom_side: 2,
            left_side: 8,
        },
        Piece {
            piece_number: 92,
            top_side: 19,
            right_side: 15,
            bottom_side: 2,
            left_side: 8,
        },
        Piece {
            piece_number: 93,
            top_side: 19,
            right_side: 4,
            bottom_side: 2,
            left_side: 8,
        },
        Piece {
            piece_number: 94,
            top_side: 4,
            right_side: 21,
            bottom_side: 2,
            left_side: 8,
        },
        Piece {
            piece_number: 95,
            top_side: 12,
            right_side: 14,
            bottom_side: 2,
            left_side: 8,
        },
        Piece {
            piece_number: 96,
            top_side: 21,
            right_side: 3,
            bottom_side: 2,
            left_side: 21,
        },
        Piece {
            piece_number: 97,
            top_side: 4,
            right_side: 19,
            bottom_side: 2,
            left_side: 22,
        },
        Piece {
            piece_number: 98,
            top_side: 20,
            right_side: 8,
            bottom_side: 2,
            left_side: 22,
        },
        Piece {
            piece_number: 99,
            top_side: 21,
            right_side: 6,
            bottom_side: 2,
            left_side: 22,
        },
        Piece {
            piece_number: 100,
            top_side: 22,
            right_side: 21,
            bottom_side: 2,
            left_side: 22,
        },
        Piece {
            piece_number: 101,
            top_side: 12,
            right_side: 15,
            bottom_side: 10,
            left_side: 10,
        },
        Piece {
            piece_number: 102,
            top_side: 12,
            right_side: 16,
            bottom_side: 10,
            left_side: 10,
        },
        Piece {
            piece_number: 103,
            top_side: 16,
            right_side: 19,
            bottom_side: 10,
            left_side: 10,
        },
        Piece {
            piece_number: 104,
            top_side: 22,
            right_side: 6,
            bottom_side: 10,
            left_side: 10,
        },
        Piece {
            piece_number: 105,
            top_side: 4,
            right_side: 15,
            bottom_side: 10,
            left_side: 18,
        },
        Piece {
            piece_number: 106,
            top_side: 3,
            right_side: 8,
            bottom_side: 10,
            left_side: 6,
        },
        Piece {
            piece_number: 107,
            top_side: 19,
            right_side: 8,
            bottom_side: 10,
            left_side: 6,
        },
        Piece {
            piece_number: 108,
            top_side: 4,
            right_side: 15,
            bottom_side: 10,
            left_side: 6,
        },
        Piece {
            piece_number: 109,
            top_side: 16,
            right_side: 11,
            bottom_side: 10,
            left_side: 6,
        },
        Piece {
            piece_number: 110,
            top_side: 15,
            right_side: 12,
            bottom_side: 10,
            left_side: 14,
        },
        Piece {
            piece_number: 111,
            top_side: 12,
            right_side: 15,
            bottom_side: 10,
            left_side: 14,
        },
        Piece {
            piece_number: 112,
            top_side: 20,
            right_side: 19,
            bottom_side: 10,
            left_side: 3,
        },
        Piece {
            piece_number: 113,
            top_side: 20,
            right_side: 16,
            bottom_side: 10,
            left_side: 3,
        },
        Piece {
            piece_number: 114,
            top_side: 14,
            right_side: 4,
            bottom_side: 10,
            left_side: 11,
        },
        Piece {
            piece_number: 115,
            top_side: 7,
            right_side: 12,
            bottom_side: 10,
            left_side: 11,
        },
        Piece {
            piece_number: 116,
            top_side: 12,
            right_side: 11,
            bottom_side: 10,
            left_side: 11,
        },
        Piece {
            piece_number: 117,
            top_side: 22,
            right_side: 16,
            bottom_side: 10,
            left_side: 11,
        },
        Piece {
            piece_number: 118,
            top_side: 3,
            right_side: 21,
            bottom_side: 10,
            left_side: 19,
        },
        Piece {
            piece_number: 119,
            top_side: 16,
            right_side: 12,
            bottom_side: 10,
            left_side: 7,
        },
        Piece {
            piece_number: 120,
            top_side: 8,
            right_side: 22,
            bottom_side: 10,
            left_side: 15,
        },
        Piece {
            piece_number: 121,
            top_side: 14,
            right_side: 22,
            bottom_side: 10,
            left_side: 4,
        },
        Piece {
            piece_number: 122,
            top_side: 6,
            right_side: 16,
            bottom_side: 10,
            left_side: 20,
        },
        Piece {
            piece_number: 123,
            top_side: 14,
            right_side: 19,
            bottom_side: 10,
            left_side: 20,
        },
        Piece {
            piece_number: 124,
            top_side: 20,
            right_side: 15,
            bottom_side: 10,
            left_side: 20,
        },
        Piece {
            piece_number: 125,
            top_side: 12,
            right_side: 22,
            bottom_side: 10,
            left_side: 8,
        },
        Piece {
            piece_number: 126,
            top_side: 21,
            right_side: 15,
            bottom_side: 10,
            left_side: 8,
        },
        Piece {
            piece_number: 127,
            top_side: 14,
            right_side: 6,
            bottom_side: 10,
            left_side: 16,
        },
        Piece {
            piece_number: 128,
            top_side: 19,
            right_side: 21,
            bottom_side: 10,
            left_side: 16,
        },
        Piece {
            piece_number: 129,
            top_side: 4,
            right_side: 3,
            bottom_side: 10,
            left_side: 16,
        },
        Piece {
            piece_number: 130,
            top_side: 20,
            right_side: 8,
            bottom_side: 10,
            left_side: 16,
        },
        Piece {
            piece_number: 131,
            top_side: 6,
            right_side: 20,
            bottom_side: 10,
            left_side: 21,
        },
        Piece {
            piece_number: 132,
            top_side: 12,
            right_side: 14,
            bottom_side: 10,
            left_side: 21,
        },
        Piece {
            piece_number: 133,
            top_side: 14,
            right_side: 16,
            bottom_side: 10,
            left_side: 22,
        },
        Piece {
            piece_number: 134,
            top_side: 11,
            right_side: 4,
            bottom_side: 10,
            left_side: 22,
        },
        Piece {
            piece_number: 135,
            top_side: 4,
            right_side: 3,
            bottom_side: 10,
            left_side: 22,
        },
        Piece {
            piece_number: 136,
            top_side: 16,
            right_side: 20,
            bottom_side: 10,
            left_side: 22,
        },
        Piece {
            piece_number: 137,
            top_side: 20,
            right_side: 7,
            bottom_side: 18,
            left_side: 18,
        },
        Piece {
            piece_number: 138,
            top_side: 6,
            right_side: 3,
            bottom_side: 18,
            left_side: 6,
        },
        Piece {
            piece_number: 139,
            top_side: 6,
            right_side: 11,
            bottom_side: 18,
            left_side: 6,
        },
        Piece {
            piece_number: 140,
            top_side: 6,
            right_side: 12,
            bottom_side: 18,
            left_side: 6,
        },
        Piece {
            piece_number: 141,
            top_side: 19,
            right_side: 21,
            bottom_side: 18,
            left_side: 6,
        },
        Piece {
            piece_number: 142,
            top_side: 15,
            right_side: 6,
            bottom_side: 18,
            left_side: 6,
        },
        Piece {
            piece_number: 143,
            top_side: 16,
            right_side: 12,
            bottom_side: 18,
            left_side: 6,
        },
        Piece {
            piece_number: 144,
            top_side: 21,
            right_side: 21,
            bottom_side: 18,
            left_side: 6,
        },
        Piece {
            piece_number: 145,
            top_side: 3,
            right_side: 4,
            bottom_side: 18,
            left_side: 14,
        },
        Piece {
            piece_number: 146,
            top_side: 18,
            right_side: 12,
            bottom_side: 18,
            left_side: 3,
        },
        Piece {
            piece_number: 147,
            top_side: 18,
            right_side: 22,
            bottom_side: 18,
            left_side: 3,
        },
        Piece {
            piece_number: 148,
            top_side: 3,
            right_side: 14,
            bottom_side: 18,
            left_side: 3,
        },
        Piece {
            piece_number: 149,
            top_side: 15,
            right_side: 12,
            bottom_side: 18,
            left_side: 3,
        },
        Piece {
            piece_number: 150,
            top_side: 6,
            right_side: 11,
            bottom_side: 18,
            left_side: 19,
        },
        Piece {
            piece_number: 151,
            top_side: 4,
            right_side: 22,
            bottom_side: 18,
            left_side: 19,
        },
        Piece {
            piece_number: 152,
            top_side: 11,
            right_side: 11,
            bottom_side: 18,
            left_side: 7,
        },
        Piece {
            piece_number: 153,
            top_side: 11,
            right_side: 19,
            bottom_side: 18,
            left_side: 7,
        },
        Piece {
            piece_number: 154,
            top_side: 22,
            right_side: 16,
            bottom_side: 18,
            left_side: 7,
        },
        Piece {
            piece_number: 155,
            top_side: 7,
            right_side: 7,
            bottom_side: 18,
            left_side: 4,
        },
        Piece {
            piece_number: 156,
            top_side: 7,
            right_side: 12,
            bottom_side: 18,
            left_side: 4,
        },
        Piece {
            piece_number: 157,
            top_side: 22,
            right_side: 7,
            bottom_side: 18,
            left_side: 4,
        },
        Piece {
            piece_number: 158,
            top_side: 7,
            right_side: 16,
            bottom_side: 18,
            left_side: 20,
        },
        Piece {
            piece_number: 159,
            top_side: 8,
            right_side: 6,
            bottom_side: 18,
            left_side: 20,
        },
        Piece {
            piece_number: 160,
            top_side: 21,
            right_side: 21,
            bottom_side: 18,
            left_side: 8,
        },
        Piece {
            piece_number: 161,
            top_side: 6,
            right_side: 20,
            bottom_side: 18,
            left_side: 16,
        },
        Piece {
            piece_number: 162,
            top_side: 14,
            right_side: 20,
            bottom_side: 18,
            left_side: 16,
        },
        Piece {
            piece_number: 163,
            top_side: 15,
            right_side: 11,
            bottom_side: 18,
            left_side: 22,
        },
        Piece {
            piece_number: 164,
            top_side: 4,
            right_side: 16,
            bottom_side: 18,
            left_side: 22,
        },
        Piece {
            piece_number: 165,
            top_side: 3,
            right_side: 4,
            bottom_side: 6,
            left_side: 14,
        },
        Piece {
            piece_number: 166,
            top_side: 4,
            right_side: 8,
            bottom_side: 6,
            left_side: 14,
        },
        Piece {
            piece_number: 167,
            top_side: 3,
            right_side: 3,
            bottom_side: 6,
            left_side: 11,
        },
        Piece {
            piece_number: 168,
            top_side: 11,
            right_side: 15,
            bottom_side: 6,
            left_side: 19,
        },
        Piece {
            piece_number: 169,
            top_side: 19,
            right_side: 21,
            bottom_side: 6,
            left_side: 19,
        },
        Piece {
            piece_number: 170,
            top_side: 4,
            right_side: 8,
            bottom_side: 6,
            left_side: 7,
        },
        Piece {
            piece_number: 171,
            top_side: 20,
            right_side: 16,
            bottom_side: 6,
            left_side: 7,
        },
        Piece {
            piece_number: 172,
            top_side: 21,
            right_side: 11,
            bottom_side: 6,
            left_side: 7,
        },
        Piece {
            piece_number: 173,
            top_side: 15,
            right_side: 15,
            bottom_side: 6,
            left_side: 15,
        },
        Piece {
            piece_number: 174,
            top_side: 12,
            right_side: 20,
            bottom_side: 6,
            left_side: 15,
        },
        Piece {
            piece_number: 175,
            top_side: 7,
            right_side: 21,
            bottom_side: 6,
            left_side: 4,
        },
        Piece {
            piece_number: 176,
            top_side: 7,
            right_side: 19,
            bottom_side: 6,
            left_side: 12,
        },
        Piece {
            piece_number: 177,
            top_side: 14,
            right_side: 4,
            bottom_side: 6,
            left_side: 20,
        },
        Piece {
            piece_number: 178,
            top_side: 12,
            right_side: 16,
            bottom_side: 6,
            left_side: 8,
        },
        Piece {
            piece_number: 179,
            top_side: 8,
            right_side: 15,
            bottom_side: 6,
            left_side: 8,
        },
        Piece {
            piece_number: 180,
            top_side: 7,
            right_side: 16,
            bottom_side: 6,
            left_side: 16,
        },
        Piece {
            piece_number: 181,
            top_side: 11,
            right_side: 16,
            bottom_side: 6,
            left_side: 21,
        },
        Piece {
            piece_number: 182,
            top_side: 7,
            right_side: 11,
            bottom_side: 6,
            left_side: 21,
        },
        Piece {
            piece_number: 183,
            top_side: 19,
            right_side: 8,
            bottom_side: 14,
            left_side: 14,
        },
        Piece {
            piece_number: 184,
            top_side: 22,
            right_side: 7,
            bottom_side: 14,
            left_side: 3,
        },
        Piece {
            piece_number: 185,
            top_side: 19,
            right_side: 12,
            bottom_side: 14,
            left_side: 11,
        },
        Piece {
            piece_number: 186,
            top_side: 8,
            right_side: 8,
            bottom_side: 14,
            left_side: 11,
        },
        Piece {
            piece_number: 187,
            top_side: 21,
            right_side: 7,
            bottom_side: 14,
            left_side: 19,
        },
        Piece {
            piece_number: 188,
            top_side: 14,
            right_side: 21,
            bottom_side: 14,
            left_side: 7,
        },
        Piece {
            piece_number: 189,
            top_side: 3,
            right_side: 19,
            bottom_side: 14,
            left_side: 7,
        },
        Piece {
            piece_number: 190,
            top_side: 16,
            right_side: 19,
            bottom_side: 14,
            left_side: 7,
        },
        Piece {
            piece_number: 191,
            top_side: 3,
            right_side: 3,
            bottom_side: 14,
            left_side: 15,
        },
        Piece {
            piece_number: 192,
            top_side: 15,
            right_side: 20,
            bottom_side: 14,
            left_side: 15,
        },
        Piece {
            piece_number: 193,
            top_side: 11,
            right_side: 7,
            bottom_side: 14,
            left_side: 4,
        },
        Piece {
            piece_number: 194,
            top_side: 21,
            right_side: 11,
            bottom_side: 14,
            left_side: 12,
        },
        Piece {
            piece_number: 195,
            top_side: 21,
            right_side: 22,
            bottom_side: 14,
            left_side: 12,
        },
        Piece {
            piece_number: 196,
            top_side: 22,
            right_side: 15,
            bottom_side: 14,
            left_side: 12,
        },
        Piece {
            piece_number: 197,
            top_side: 11,
            right_side: 22,
            bottom_side: 14,
            left_side: 20,
        },
        Piece {
            piece_number: 198,
            top_side: 19,
            right_side: 8,
            bottom_side: 14,
            left_side: 20,
        },
        Piece {
            piece_number: 199,
            top_side: 20,
            right_side: 20,
            bottom_side: 14,
            left_side: 20,
        },
        Piece {
            piece_number: 200,
            top_side: 19,
            right_side: 3,
            bottom_side: 14,
            left_side: 8,
        },
        Piece {
            piece_number: 201,
            top_side: 21,
            right_side: 8,
            bottom_side: 14,
            left_side: 16,
        },
        Piece {
            piece_number: 202,
            top_side: 22,
            right_side: 7,
            bottom_side: 14,
            left_side: 16,
        },
        Piece {
            piece_number: 203,
            top_side: 12,
            right_side: 19,
            bottom_side: 14,
            left_side: 21,
        },
        Piece {
            piece_number: 204,
            top_side: 12,
            right_side: 8,
            bottom_side: 14,
            left_side: 21,
        },
        Piece {
            piece_number: 205,
            top_side: 16,
            right_side: 3,
            bottom_side: 14,
            left_side: 21,
        },
        Piece {
            piece_number: 206,
            top_side: 22,
            right_side: 21,
            bottom_side: 14,
            left_side: 21,
        },
        Piece {
            piece_number: 207,
            top_side: 22,
            right_side: 7,
            bottom_side: 3,
            left_side: 3,
        },
        Piece {
            piece_number: 208,
            top_side: 19,
            right_side: 22,
            bottom_side: 3,
            left_side: 11,
        },
        Piece {
            piece_number: 209,
            top_side: 8,
            right_side: 15,
            bottom_side: 3,
            left_side: 11,
        },
        Piece {
            piece_number: 210,
            top_side: 11,
            right_side: 19,
            bottom_side: 3,
            left_side: 7,
        },
        Piece {
            piece_number: 211,
            top_side: 16,
            right_side: 15,
            bottom_side: 3,
            left_side: 7,
        },
        Piece {
            piece_number: 212,
            top_side: 3,
            right_side: 16,
            bottom_side: 3,
            left_side: 15,
        },
        Piece {
            piece_number: 213,
            top_side: 8,
            right_side: 8,
            bottom_side: 3,
            left_side: 4,
        },
        Piece {
            piece_number: 214,
            top_side: 3,
            right_side: 20,
            bottom_side: 3,
            left_side: 12,
        },
        Piece {
            piece_number: 215,
            top_side: 4,
            right_side: 22,
            bottom_side: 3,
            left_side: 12,
        },
        Piece {
            piece_number: 216,
            top_side: 22,
            right_side: 21,
            bottom_side: 3,
            left_side: 12,
        },
        Piece {
            piece_number: 217,
            top_side: 19,
            right_side: 15,
            bottom_side: 3,
            left_side: 20,
        },
        Piece {
            piece_number: 218,
            top_side: 4,
            right_side: 12,
            bottom_side: 3,
            left_side: 16,
        },
        Piece {
            piece_number: 219,
            top_side: 11,
            right_side: 4,
            bottom_side: 3,
            left_side: 21,
        },
        Piece {
            piece_number: 220,
            top_side: 11,
            right_side: 16,
            bottom_side: 3,
            left_side: 22,
        },
        Piece {
            piece_number: 221,
            top_side: 21,
            right_side: 21,
            bottom_side: 3,
            left_side: 22,
        },
        Piece {
            piece_number: 222,
            top_side: 21,
            right_side: 22,
            bottom_side: 3,
            left_side: 22,
        },
        Piece {
            piece_number: 223,
            top_side: 12,
            right_side: 22,
            bottom_side: 11,
            left_side: 11,
        },
        Piece {
            piece_number: 224,
            top_side: 20,
            right_side: 7,
            bottom_side: 11,
            left_side: 11,
        },
        Piece {
            piece_number: 225,
            top_side: 16,
            right_side: 15,
            bottom_side: 11,
            left_side: 11,
        },
        Piece {
            piece_number: 226,
            top_side: 19,
            right_side: 15,
            bottom_side: 11,
            left_side: 7,
        },
        Piece {
            piece_number: 227,
            top_side: 12,
            right_side: 12,
            bottom_side: 11,
            left_side: 7,
        },
        Piece {
            piece_number: 228,
            top_side: 19,
            right_side: 8,
            bottom_side: 11,
            left_side: 4,
        },
        Piece {
            piece_number: 229,
            top_side: 7,
            right_side: 22,
            bottom_side: 11,
            left_side: 20,
        },
        Piece {
            piece_number: 230,
            top_side: 16,
            right_side: 8,
            bottom_side: 11,
            left_side: 20,
        },
        Piece {
            piece_number: 231,
            top_side: 12,
            right_side: 20,
            bottom_side: 11,
            left_side: 8,
        },
        Piece {
            piece_number: 232,
            top_side: 12,
            right_side: 21,
            bottom_side: 11,
            left_side: 8,
        },
        Piece {
            piece_number: 233,
            top_side: 19,
            right_side: 20,
            bottom_side: 19,
            left_side: 19,
        },
        Piece {
            piece_number: 234,
            top_side: 16,
            right_side: 4,
            bottom_side: 19,
            left_side: 7,
        },
        Piece {
            piece_number: 235,
            top_side: 7,
            right_side: 4,
            bottom_side: 19,
            left_side: 4,
        },
        Piece {
            piece_number: 236,
            top_side: 7,
            right_side: 20,
            bottom_side: 19,
            left_side: 4,
        },
        Piece {
            piece_number: 237,
            top_side: 12,
            right_side: 15,
            bottom_side: 19,
            left_side: 4,
        },
        Piece {
            piece_number: 238,
            top_side: 4,
            right_side: 16,
            bottom_side: 19,
            left_side: 12,
        },
        Piece {
            piece_number: 239,
            top_side: 15,
            right_side: 22,
            bottom_side: 19,
            left_side: 20,
        },
        Piece {
            piece_number: 240,
            top_side: 21,
            right_side: 15,
            bottom_side: 19,
            left_side: 20,
        },
        Piece {
            piece_number: 241,
            top_side: 7,
            right_side: 21,
            bottom_side: 19,
            left_side: 8,
        },
        Piece {
            piece_number: 242,
            top_side: 4,
            right_side: 21,
            bottom_side: 19,
            left_side: 8,
        },
        Piece {
            piece_number: 243,
            top_side: 15,
            right_side: 12,
            bottom_side: 7,
            left_side: 15,
        },
        Piece {
            piece_number: 244,
            top_side: 20,
            right_side: 8,
            bottom_side: 7,
            left_side: 15,
        },
        Piece {
            piece_number: 245,
            top_side: 22,
            right_side: 20,
            bottom_side: 7,
            left_side: 4,
        },
        Piece {
            piece_number: 246,
            top_side: 16,
            right_side: 22,
            bottom_side: 7,
            left_side: 21,
        },
        Piece {
            piece_number: 247,
            top_side: 21,
            right_side: 22,
            bottom_side: 15,
            left_side: 15,
        },
        Piece {
            piece_number: 248,
            top_side: 12,
            right_side: 4,
            bottom_side: 15,
            left_side: 4,
        },
        Piece {
            piece_number: 249,
            top_side: 4,
            right_side: 21,
            bottom_side: 15,
            left_side: 12,
        },
        Piece {
            piece_number: 250,
            top_side: 16,
            right_side: 21,
            bottom_side: 15,
            left_side: 20,
        },
        Piece {
            piece_number: 251,
            top_side: 22,
            right_side: 8,
            bottom_side: 4,
            left_side: 4,
        },
        Piece {
            piece_number: 252,
            top_side: 8,
            right_side: 12,
            bottom_side: 4,
            left_side: 12,
        },
        Piece {
            piece_number: 253,
            top_side: 16,
            right_side: 20,
            bottom_side: 12,
            left_side: 8,
        },
        Piece {
            piece_number: 254,
            top_side: 21,
            right_side: 16,
            bottom_side: 20,
            left_side: 16,
        },
        Piece {
            piece_number: 255,
            top_side: 16,
            right_side: 22,
            bottom_side: 20,
            left_side: 22,
        },
        Piece {
            piece_number: 256,
            top_side: 21,
            right_side: 22,
            bottom_side: 8,
            left_side: 22,
        },
    ]
}
