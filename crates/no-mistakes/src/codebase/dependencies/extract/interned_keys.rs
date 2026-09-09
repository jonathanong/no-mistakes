//! Construction lock: ImportCollector interned keys stay on crate::fx nested maps.

#[test]
fn import_collector_nests_interned_keys_on_fxhash() {
    let extract = include_str!("../extract.rs");
    let collector = include_str!("../extract_collector.rs");
    let maps = include_str!("../extract_collector_maps.rs");
    let scopes = include_str!("../extract_collector_scopes.rs");

    assert!(
        extract.contains("use crate::fx::{fx_map, fx_set, FxHashMap, FxHashSet}"),
        "extract collector must import crate::fx maps"
    );
    assert!(
        !extract.contains("std::collections::HashMap"),
        "extract collector must not import SipHash HashMap"
    );
    assert!(
        !extract.contains("std::collections::HashSet"),
        "extract collector must not import SipHash HashSet"
    );
    assert!(
        collector.contains("callable_bindings: Vec<FxHashMap<String, CallableId>>"),
        "lexical bindings must nest per scope so lookups use &str"
    );
    assert!(
        collector.contains(
            "class_member_callable_ids: FxHashMap<CallableId, FxHashMap<String, CallableId>>"
        ),
        "class members must nest by owner so (class, member) probes skip a linear scan"
    );
    assert!(
        !collector.contains("HashMap<(usize, String)"),
        "do not key collector maps on allocated (scope, String) tuples"
    );
    assert!(
        maps.contains("fn scope_map_get"),
        "name lookups must go through borrowed &str helpers"
    );
    assert!(
        !scopes.contains("HashSet::new()"),
        "rustc-hash 2 FxHashSet has no new(); use fx_set()"
    );
}
