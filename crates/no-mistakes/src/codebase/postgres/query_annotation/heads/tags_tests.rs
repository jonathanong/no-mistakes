use super::support::{assert_markers_for, decisions_for, fixture, recollect};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::codebase::postgres::{EmbeddedSqlOptions, TrustedSqlTag};
use std::sync::Arc;
fn options(module: &str, name: &str) -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@bench/db", &[]).with_trusted_sql_tags(&[TrustedSqlTag {
        module: module.into(),
        name: name.into(),
    }])
}
#[test]
fn configured_named_tags_use_the_actual_binding_and_first_quasi() {
    let (source, _, heads) = fixture("tags.mts");
    assert!(assert_markers_for(&source, &heads, &options("@bench/tags", "sql")) >= 20);
    let shared = heads
        .calls
        .values()
        .filter_map(|proof| match proof {
            HeadProof::NamedImport(actual) if actual.module == "@bench/tags" => Some(actual),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        shared.len() >= 4,
        "direct/one-const/captured proofs retain import identity"
    );
    assert!(shared.iter().all(|actual| Arc::ptr_eq(actual, shared[0])));
}
#[test]
fn the_same_facts_do_not_share_trust_between_profiles_or_builtin_tags() {
    let (source, _, heads) = fixture("tags.mts");
    for options in [
        EmbeddedSqlOptions::default(),
        options("", "sql"),
        options("@bench/tags", ""),
        options("@bench/tags", "named"),
    ] {
        let accepted = decisions_for(&source, &heads, &options)
            .into_iter()
            .filter(|(_, accepted)| *accepted)
            .collect::<Vec<_>>();
        assert_eq!(
            accepted.len(),
            1,
            "only the ordinary literal is unconditional"
        );
    }
    for (profile, expression) in [
        (options("@other/tags", "sql"), "read(other`"),
        (options("sql-template-strings", "sql"), "read(builtinNamed`"),
    ] {
        let accepted = decisions_for(&source, &heads, &profile)
            .into_iter()
            .filter(|(_, accepted)| *accepted)
            .collect::<Vec<_>>();
        assert_eq!(accepted.len(), 2);
        assert!(accepted
            .iter()
            .any(|(span, _)| source[span.0 as usize..].starts_with(expression)));
    }
    let profile = options("sql-template-strings", "default");
    assert_eq!(
        decisions_for(&source, &heads, &profile)
            .iter()
            .filter(|(_, accepted)| *accepted)
            .count(),
        1
    );
}
#[test]
fn tag_ownership_fails_closed_for_missing_ambiguous_and_diagnostic_facts() {
    let (source, original, _) = fixture("tags.mts");
    let profile = options("@bench/tags", "sql");
    let mut missing = original.clone();
    missing.module_bindings = None;
    let heads = recollect("tags.mts", &missing);
    assert_eq!(
        decisions_for(&source, &heads, &profile)
            .iter()
            .filter(|(_, accepted)| *accepted)
            .count(),
        1
    );
    let mut duplicate = original.clone();
    let module = Arc::make_mut(duplicate.module_bindings.as_mut().unwrap());
    let mut other = module
        .bindings
        .iter()
        .find(|binding| binding.name == "named" && binding.imported)
        .unwrap()
        .clone();
    other.imported = false;
    module.bindings.push(other);
    let heads = recollect("tags.mts", &duplicate);
    assert!(decisions_for(&source, &heads, &profile)
        .iter()
        .filter(|(span, _)| source[span.0 as usize..].starts_with("read(named`"))
        .all(|(_, accepted)| !accepted));
    let mut diagnostic = original;
    Arc::make_mut(diagnostic.module_bindings.as_mut().unwrap())
        .diagnostics
        .push(crate::codebase::ts_source::module_facts::ModuleDiagnostic {
            kind: "unsupported".into(),
            message: "Dynamic eval may change binding semantics".into(),
            span: Some(crate::codebase::ts_source::module_facts::ModuleSpan { start: 0, end: 1 }),
        });
    let heads = recollect("tags.mts", &diagnostic);
    assert_eq!(
        decisions_for(&source, &heads, &profile)
            .iter()
            .filter(|(_, accepted)| *accepted)
            .count(),
        1
    );
}

#[test]
fn tag_constants_reject_mutations_and_escapes_but_allow_executor_reads() {
    let (source, _, heads) = fixture("tag-const-effects.mts");
    assert_eq!(
        assert_markers_for(&source, &heads, &options("@bench/tags", "sql")),
        14
    );
}
