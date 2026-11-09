use super::*;

#[test]
fn imported_storage_constructor_does_not_execute_instance_callbacks() {
    let source = proof_source("proof-storage-constructor.mts");
    let facts = proven_facts(&source);
    for call in ["Store.run", "Store.exit"] {
        assert!(callback_values(&source, call, &facts).is_empty(), "{call}");
        assert!(
            !facts.calls.contains(&(source.find(call).unwrap() as u32)),
            "{call}"
        );
    }
    for (call, position) in [
        ("instance.run", 1),
        ("instance.exit", 0),
        ("new Store<string>().run", 1),
        ("typed.run", 1),
    ] {
        assert_eq!(
            callback_values(&source, call, &facts),
            vec![Some(vec![position])],
            "{call}"
        );
    }
}
