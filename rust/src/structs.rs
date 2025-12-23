#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub reid: u16, // Real ID (1-256).
    pub top: u8,
    pub right: u8,
    pub bottom: u8,
    pub left: u8,
}

impl Piece {
    /// Returns the type of piece: 2 for corners, 1 for sides, and 0 for middles
    pub fn piece_type(&self) -> u8 {
        match self.reid {
            1..=4 => 2,
            5..=60 => 1,
            _ => 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RotatedPiece {
    pub reid: u16,
    pub rotations: u8,
    pub top: u8,
    pub right: u8,
    pub breaks: u8,
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
