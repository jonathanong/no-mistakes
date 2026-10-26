use super::Positions;

impl Default for Positions {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            line: 1,
            column: 1,
            origin: 1,
            placeholder_offset: 0,
        }
    }
}
