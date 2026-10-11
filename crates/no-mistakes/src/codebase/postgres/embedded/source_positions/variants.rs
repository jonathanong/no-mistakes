use super::{EmbeddedSqlSourcePosition, Positions};

pub(in crate::codebase::postgres::embedded) fn piece(
    raw: &str,
    decoded: &str,
    line: u32,
    raw_mode: bool,
) -> Vec<EmbeddedSqlSourcePosition> {
    let mut positions = Positions {
        origin: line,
        ..Positions::default()
    };
    positions.append(raw, decoded, line, raw_mode);
    positions.positions
}
