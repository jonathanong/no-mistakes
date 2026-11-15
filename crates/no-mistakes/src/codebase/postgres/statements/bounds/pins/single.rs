//! `=` to one caller value. `IN`, `ANY`, and `IS NOT DISTINCT FROM` stay multi-valued.
use crate::codebase::postgres::statements::{SqlBoundPin, SqlPinSource};

pub(super) fn mark(pins: &mut [(usize, SqlBoundPin)]) {
    for (_, pin) in pins {
        pin.single_value = matches!(pin.source, SqlPinSource::Value)
            && pin.reads.is_empty()
            && pin.qualified_reads.is_empty()
            && !pin.null_safe
            && !pin.unqualified_candidate
            && pin.expression.is_none()
            && pin.alternative.is_none();
    }
}
