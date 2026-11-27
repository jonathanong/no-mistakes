use crate::fx::FxHashMap;
use oxc_syntax::symbol::SymbolId;

#[derive(Default)]
pub(in super::super) struct AliasSafety {
    pub(super) entries: FxHashMap<SymbolId, bool>,
}

// No expression, field or call survives in a production projection.
macro_rules! record_reference_scan {
    ($cache:expr) => {};
}
