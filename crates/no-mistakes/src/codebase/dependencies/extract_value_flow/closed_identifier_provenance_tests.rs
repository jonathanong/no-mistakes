use super::super::{unique_closed_collection, GuardKind, RegistryKind};
use super::parse_flow;

#[test]
fn erased_imported_catalog_type_reads_do_not_escape_the_runtime_collection() {
    let flow = parse_flow(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-imported-guards/save.ts")), true);
    assert!(!flow
        .registry_mutations
        .iter()
        .any(|mutation| mutation.name == "ALLOWED_TABLES"));
    assert_eq!(flow.finite_identifier_guards.iter().filter(|guard| {
        matches!(&guard.kind, crate::codebase::dependencies::extract::GuardKind::ClosedSet { collection, .. } if collection == "ALLOWED_TABLES")
    }).count(), 2);
}

#[test]
fn property_guards_retain_the_exact_lexical_subject_and_member_path() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards/src/guarded.ts"));
    let flow = parse_flow(source, true);
    let members: Vec<_> = flow
        .finite_identifier_guards
        .iter()
        .filter(|guard| guard.name == "config")
        .map(|guard| guard.members.join("."))
        .collect();
    assert_eq!(members, ["schema", "table", "table", "schema", "table"]);
    assert!(flow.finite_identifier_guards.iter().any(|guard| guard.name == "table" && matches!(&guard.kind, GuardKind::Assertion { callee, .. } if callee == "validators.validateIdentifier")));
}

#[test]
fn metadata_lookup_facts_keep_closed_fields_and_reject_open_or_aliased_arrays() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards/src/metadata.ts"));
    let flow = parse_flow(source, true);
    assert!(unique_closed_collection(
        &flow,
        "TABLES",
        RegistryKind::MetadataArray
    ));
    assert!(!unique_closed_collection(
        &flow,
        "OPEN_TABLES",
        RegistryKind::MetadataArray
    ));
    assert!(!unique_closed_collection(
        &flow,
        "EFFECT_TABLES",
        RegistryKind::MetadataArray
    ));
    assert!(!unique_closed_collection(
        &flow,
        "ESCAPED_TABLES",
        RegistryKind::MetadataArray
    ));
    let proven: Vec<_> = flow
        .bounded_bindings
        .iter()
        .filter(|binding| {
            binding
                .source
                .as_ref()
                .is_some_and(|source| source.0 == "TABLES")
        })
        .map(|binding| binding.members.join("."))
        .collect();
    assert_eq!(
        proven,
        ["name", "schema", "table", "name", "schema", "table"]
    );
    assert_eq!(
        flow.finite_identifier_guards
            .iter()
            .filter(|guard| matches!(guard.kind, GuardKind::Lookup))
            .count(),
        3
    );
}

#[test]
fn getter_member_validation_never_establishes_a_guard_subject() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards/src/getters.ts"));
    let flow = parse_flow(source, true);
    let guards: Vec<_> = flow
        .finite_identifier_guards
        .iter()
        .filter(|guard| guard.name == "config")
        .collect();
    assert_eq!(guards.len(), 1);
    assert_eq!(guards[0].members, ["table"]);
    assert_eq!(guards[0].subject, None);
}

#[test]
fn finite_registries_require_unshadowed_builtin_constructors() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards/src/unguarded.ts"));
    let flow = parse_flow(source, true);
    for name in ["SHADOWED_TABLES", "SHADOWED_FREEZE", "SHADOWED_MAP"] {
        assert!(
            !flow
                .closed_string_registries
                .iter()
                .any(|registry| registry.name == name),
            "{name}"
        );
    }
}

#[test]
fn placeholder_lists_decline_non_integer_interpolations() {
    let flow = parse_flow(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/codebase-analysis/closed-sql-identifier-domains/placeholder-boundary.ts")), true);
    assert!(flow
        .sql_placeholder_lists
        .iter()
        .any(|(_, name, _)| name == "valid"));
    assert!(!flow
        .sql_placeholder_lists
        .iter()
        .any(|(_, name, _)| name == "invalid"));
    assert!(flow
        .bounded_bindings
        .iter()
        .any(|binding| binding.name == "validJoined"));
    assert!(!flow
        .bounded_bindings
        .iter()
        .any(|binding| binding.name == "invalidJoined"));
}
