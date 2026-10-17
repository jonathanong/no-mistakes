use crate::fx::FxHashMap;
use oxc_syntax::symbol::SymbolId;

#[derive(Default)]
pub(in super::super) struct AliasSafety {
    pub(super) entries: FxHashMap<SymbolId, bool>,
    pub(super) scanned_references: usize,
}

macro_rules! record_reference_scan {
    ($cache:expr) => {
        $cache.scanned_references += 1;
    };
}
