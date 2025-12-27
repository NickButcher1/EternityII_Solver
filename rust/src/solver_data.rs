use crate::board_order::get_board_order;
use crate::config::BREAK_INDEXES_ALLOWED;
use crate::config::{HEURISTIC_SIDES, MAX_HEURISTIC_INDEX};
use crate::pieces;
use crate::structs::{Piece, RotatedPiece, RotatedPieceWithLeftBottom, SearchIndex};
use rand::Rng;
use std::collections::HashMap;

const SIDE_EDGES: &[u8] = &[1, 5, 9, 13, 17];

#[derive(Debug, Clone, Copy)]
pub enum PieceCategory {
    None,
    Corners,
    LeftSides,
    RightSidesWithBreaks,
    RightSidesWithoutBreaks,
    TopSides,
    MiddlesWithBreak,
    MiddlesNoBreak,
    SouthStart,
    WestStart,
    Start,
}

pub struct SolverData {
    pub no_pieces: Vec<Vec<RotatedPiece>>,
    pub corners: Vec<Vec<RotatedPiece>>,
    left_sides: Vec<Vec<RotatedPiece>>,
    right_sides_with_breaks: Vec<Vec<RotatedPiece>>,
    right_sides_without_breaks: Vec<Vec<RotatedPiece>>,
    top_sides: Vec<Vec<RotatedPiece>>,
    middles_with_break: Vec<Vec<RotatedPiece>>,
    middles_no_break: Vec<Vec<RotatedPiece>>,
    south_start: Vec<Vec<RotatedPiece>>,
    west_start: Vec<Vec<RotatedPiece>>,
    start: Vec<Vec<RotatedPiece>>,
    pub bottom_side_pieces_rotated: HashMap<u16, Vec<RotatedPieceWithLeftBottom>>,
    pub master_piece_lookup: [PieceCategory; 256],
    pub board_search_sequence: [SearchIndex; 256],
    pub break_array: [u8; 256],
    pub heuristic_array: Vec<i32>,
}

impl SolverData {
    pub fn get_pieces(&self, category: PieceCategory) -> &[Vec<RotatedPiece>] {
        match category {
            PieceCategory::None => &self.no_pieces,
            PieceCategory::Corners => &self.corners,
            PieceCategory::LeftSides => &self.left_sides,
            PieceCategory::RightSidesWithBreaks => &self.right_sides_with_breaks,
            PieceCategory::RightSidesWithoutBreaks => &self.right_sides_without_breaks,
            PieceCategory::TopSides => &self.top_sides,
            PieceCategory::MiddlesWithBreak => &self.middles_with_break,
            PieceCategory::MiddlesNoBreak => &self.middles_no_break,
            PieceCategory::SouthStart => &self.south_start,
            PieceCategory::WestStart => &self.west_start,
            PieceCategory::Start => &self.start,
        }
    }
}

fn calculate_two_sides(side1: u16, side2: u16) -> u16 {
    (side1 * 23) + side2
}

fn get_rotated_pieces(piece: &Piece, allow_breaks: bool) -> Vec<RotatedPieceWithLeftBottom> {
    let mut score_base: i32 = 0;
    let mut heuristic_side_count: u8 = 0;

    // Calculate heuristic score
    for &side in HEURISTIC_SIDES {
        if piece.left == side {
            score_base += 100;
            heuristic_side_count += 1;
        }
        if piece.top == side {
            score_base += 100;
            heuristic_side_count += 1;
        }
        if piece.right == side {
            score_base += 100;
            heuristic_side_count += 1;
        }
        if piece.bottom == side {
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
                piece.left,
                piece.bottom,
                piece.top,
                piece.right,
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
                piece.bottom,
                piece.right,
                piece.left,
                piece.top,
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
                piece.right,
                piece.top,
                piece.bottom,
                piece.left,
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
                piece.top,
                piece.left,
                piece.right,
                piece.bottom,
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
                reid: piece.reid,
                rotations: rot_idx,
                top: out_top,
                right: out_right,
                breaks,
                heuristic_side_count: h_count,
            },
        });
    }
}

fn first_break_index() -> usize {
    *BREAK_INDEXES_ALLOWED.iter().min().unwrap_or(&256)
}

fn get_break_array() -> [u8; 256] {
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

fn get_heuristic_array() -> Vec<i32> {
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
    heuristic_array
}

pub fn prepare_pieces_and_heuristics() -> SolverData {
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
            .flat_map(|x| get_rotated_pieces(x, false))
            .collect(),
    );

    // Sides
    let sides_without_breaks: Vec<_> = side_pieces
        .iter()
        .flat_map(|x| get_rotated_pieces(x, false))
        .collect();

    let sides_with_breaks: Vec<_> = side_pieces
        .iter()
        .flat_map(|x| get_rotated_pieces(x, true))
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
            .flat_map(|x| get_rotated_pieces(x, true))
            .collect(),
    );

    let middle_pieces_rotated_without_breaks = group_by_left_bottom(
        middle_pieces
            .iter()
            .flat_map(|x| get_rotated_pieces(x, false))
            .collect(),
    );

    let south_start_piece_rotated = group_by_left_bottom(
        middle_pieces
            .iter()
            .flat_map(|x| get_rotated_pieces(x, false))
            .filter(|x| x.rotated_piece.top == 6)
            .collect(),
    );

    let west_start_piece_rotated = group_by_left_bottom(
        middle_pieces
            .iter()
            .flat_map(|x| get_rotated_pieces(x, false))
            .filter(|x| x.rotated_piece.right == 11)
            .collect(),
    );

    let start_piece_rotated = group_by_left_bottom(
        start_piece
            .iter()
            .flat_map(|x| get_rotated_pieces(x, false))
            .filter(|x| x.rotated_piece.rotations == 2)
            .collect(),
    );

    let mut rng = rand::rng();

    let no_pieces: Vec<Vec<RotatedPiece>> = vec![];
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
    let break_array = get_break_array();

    let mut master_piece_lookup: [PieceCategory; 256] = [PieceCategory::None; 256];

    #[allow(clippy::needless_range_loop)]
    for i in 0..256 {
        let row = board_search_sequence[i].row as usize;
        let col = board_search_sequence[i].column as usize;

        let lookup = if row == 15 {
            if col == 15 || col == 0 {
                PieceCategory::Corners
            } else {
                PieceCategory::TopSides
            }
        } else if row == 0 {
            PieceCategory::None
        } else if col == 15 {
            if i < first_break_index() {
                PieceCategory::RightSidesWithoutBreaks
            } else {
                PieceCategory::RightSidesWithBreaks
            }
        } else if col == 0 {
            PieceCategory::LeftSides
        } else if row == 7 {
            if col == 7 {
                PieceCategory::Start
            } else if col == 6 {
                PieceCategory::WestStart
            } else if i < first_break_index() {
                PieceCategory::MiddlesNoBreak
            } else {
                PieceCategory::MiddlesWithBreak
            }
        } else if row == 6 {
            if col == 7 {
                PieceCategory::SouthStart
            } else if i < first_break_index() {
                PieceCategory::MiddlesNoBreak
            } else {
                PieceCategory::MiddlesWithBreak
            }
        } else if i < first_break_index() {
            PieceCategory::MiddlesNoBreak
        } else {
            PieceCategory::MiddlesWithBreak
        };

        master_piece_lookup[row * 16 + col] = lookup;
    }

    let heuristic_array = get_heuristic_array();

    SolverData {
        no_pieces,
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
) -> Vec<Vec<RotatedPiece>> {
    let mut result = vec![Vec::new(); 529];

    for (key, value) in map {
        let mut pieces: Vec<(RotatedPiece, i32)> = value
            .iter()
            .map(|x| (x.rotated_piece, x.score + rng.random_range(0..99)))
            .collect();

        pieces.sort_by(|a, b| b.1.cmp(&a.1));

        result[*key as usize] = pieces.into_iter().map(|(p, _)| p).collect();
    }
    result
}
