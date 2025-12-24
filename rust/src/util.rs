use crate::pieces;
use crate::structs::RotatedPiece;
use std::fs;

pub fn save_board(board: &[RotatedPiece; 256], max_solve_index: u16) {
    let board_pieces = pieces::PIECES;
    let mut entire_board = String::new();
    let mut url_path = String::new();

    for i in (0..16).rev() {
        let mut row_str = String::new();
        for j in 0..16 {
            let p_rotated = board[i * 16 + j];
            if p_rotated.reid > 0 {
                row_str.push_str(&format!("{:>3}/{} ", p_rotated.reid, p_rotated.rotations));

                // Find original piece to get sides for URL
                if let Some(p) = board_pieces.iter().find(|k| k.reid == p_rotated.reid) {
                    let (t, r, b, l) = match p_rotated.rotations {
                        0 => (p.top, p.right, p.bottom, p.left),
                        1 => (p.left, p.top, p.right, p.bottom),
                        2 => (p.bottom, p.left, p.top, p.right),
                        _ => (p.right, p.bottom, p.left, p.top),
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
