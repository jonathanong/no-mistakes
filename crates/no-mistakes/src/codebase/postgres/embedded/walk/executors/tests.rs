use super::super::super::EmbeddedSqlKind;
use super::super::BindingState;

// Keep the exact pre-proof field list: a scalar proof must not silently grow
// every SQL binding or add a heap allocation.
#[allow(dead_code)]
struct LegacyBindingState {
    builder_identity: Option<u32>,
    builder_effect_revision: u64,
    condition_key: Option<(u64, u32)>,
    variants: Option<Vec<super::super::variants::Recovered>>,
    sql: Option<String>,
    kind: EmbeddedSqlKind,
    line: u32,
    initialized: bool,
    sql_builder: bool,
    sql_source_positions: Vec<super::super::super::super::EmbeddedSqlSourcePosition>,
}

#[test]
fn executor_alias_proof_preserves_binding_layout() {
    let old = std::mem::size_of::<LegacyBindingState>();
    let new = std::mem::size_of::<BindingState>();
    eprintln!("BindingState legacy={old} candidate={new}");
    assert_eq!(new, old);
}
