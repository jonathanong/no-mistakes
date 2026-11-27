use super::super::{unique_closed_collection, RegistryKind};
use super::parse_flow;

#[test]
fn immutable_row_results_produce_only_known_literal_field_facts() {
    let flow = parse_flow(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards/src/metadata-row-helpers.ts")), true);
    assert_eq!(flow.metadata_row_helpers.len(), 1);
    assert_eq!(flow.metadata_row_helpers[0].source.0, "ROWS");
    assert_eq!(
        flow.metadata_row_helpers[0].fields,
        ["key", "schema", "table"]
    );
    assert_eq!(
        flow.bounded_bindings
            .iter()
            .filter(|binding| !binding.requires_guard && !binding.members.is_empty())
            .count(),
        9
    );
    assert!(unique_closed_collection(
        &flow,
        "ROWS",
        RegistryKind::MetadataArray
    ));
}

#[test]
fn any_unsafe_helper_or_caller_dirties_every_shared_catalog_projection() {
    let flow = parse_flow(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards/src/metadata-row-unsafe.ts")), true);
    assert!(flow.metadata_row_helpers.is_empty());
    for name in [
        "MUTATION_ROWS",
        "SECOND_ROWS",
        "ALIAS_ROWS",
        "EXPORT_ROWS",
        "VALUE_ROWS",
        "VALUE_OF_ROWS",
        "RAW_RESULT_ROWS",
        "CONSTRUCTOR_ROWS",
        "CALL_ROWS",
        "TAG_ROWS",
        "REASSIGNED_ROWS",
        "FIND_ROWS",
        "UNKNOWN_ROWS",
        "SHARED_ROWS",
        "DIRECT_WRITE_ROWS",
        "ALIAS_ONLY_ROWS",
        "ALIAS_EXPORT_ROWS",
        "METHOD_ROWS",
    ] {
        assert!(
            flow.registry_mutations
                .iter()
                .any(|mutation| mutation.name == name),
            "{name}"
        );
    }
}

#[test]
fn script_functions_cannot_close_the_row_helper_call_set() {
    let flow = parse_flow(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards/src/metadata-row-script.cjs")), true);
    assert!(flow.metadata_row_helpers.is_empty());
    assert!(flow
        .registry_mutations
        .iter()
        .any(|mutation| mutation.name == "SCRIPT_ROWS"));
}

#[test]
fn prototype_initializers_never_become_literal_object_fields_but_map_keys_do() {
    let flow = parse_flow(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-cases/codebase-analysis/closed-sql-identifier-domains/proto.ts"
        )),
        true,
    );
    assert!(!flow
        .closed_string_registries
        .iter()
        .any(|registry| registry.name == "PROTO_OBJECT" || registry.name == "PROTO_METADATA"));
    assert!(unique_closed_collection(
        &flow,
        "ORDINARY_MAP",
        RegistryKind::Map
    ));
    assert!(flow
        .bounded_bindings
        .iter()
        .any(|binding| binding.name == "table"
            && binding
                .source
                .as_ref()
                .is_some_and(|source| source.0 == "ORDINARY_MAP")));
}

#[test]
fn metadata_proof_boundaries_reject_opaque_callbacks_and_unclosed_helpers() {
    let flow = parse_flow(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards/src/metadata-proof-boundaries.ts")), true);
    assert!(flow.metadata_row_helpers.is_empty());
    assert!(flow
        .bounded_bindings
        .iter()
        .any(|binding| binding.name == "reversedComparison" && binding.requires_guard));
    for name in [
        "namedCallback",
        "asyncCallback",
        "destructuredCallback",
        "looseCallback",
        "opaqueComparison",
        "computedComparison",
    ] {
        assert!(
            !flow
                .bounded_bindings
                .iter()
                .any(|binding| binding.name == name),
            "{name}"
        );
    }
    assert!(!flow
        .finite_identifier_guards
        .iter()
        .any(|guard| guard.name == "selected"
            && flow
                .parameters
                .iter()
                .any(|parameter| parameter.owner == guard.context.unwrap()
                    && parameter.name == "duplicate")));
    for source in ["MIXED_ROWS", "ASYNC_ROWS", "SPREAD_ROWS"] {
        assert!(
            flow.registry_mutations
                .iter()
                .any(|mutation| mutation.name == source),
            "{source}"
        );
    }
}
