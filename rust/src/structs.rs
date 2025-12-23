#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub piece_number: u16,
    pub top_side: u8,
    pub right_side: u8,
    pub bottom_side: u8,
    pub left_side: u8,
}

impl Piece {
    /// Returns the type of piece: 2 for corners, 1 for sides, and 0 for middles
    pub fn piece_type(&self) -> u8 {
        match self.piece_number {
            1..=4 => 2,
            5..=60 => 1,
            _ => 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RotatedPiece {
    pub piece_number: u16,
    pub rotations: u8,
    pub top_side: u8,
    pub right_side: u8,
    pub break_count: u8,
    pub heuristic_side_count: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotatedPieceWithLeftBottom {
    pub left_bottom: u16,
    pub score: i32,
    pub rotated_piece: RotatedPiece,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchIndex {
    pub row: u8,
    pub column: u8,
}
