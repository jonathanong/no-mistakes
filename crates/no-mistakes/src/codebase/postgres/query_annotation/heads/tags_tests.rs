use super::support::{assert_markers_for, decisions_for, fixture, recollect};
use crate::codebase::postgres::{EmbeddedSqlOptions, TrustedSqlTag};
fn options(module: &str, name: &str) -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@bench/db", &[]).with_trusted_sql_tags(&[TrustedSqlTag {
        module: module.into(),
        name: name.into(),
    }])
}
#[test]
fn any_tag_uses_the_first_quasi_without_import_or_profile_trust() {
    let (source, _, heads) = fixture("tags.mts");
    assert_eq!(
        assert_markers_for(&source, &heads, &EmbeddedSqlOptions::default()),
        31
    );
    let baseline = decisions_for(&source, &heads, &EmbeddedSqlOptions::default());
    for profile in [
        options("@bench/tags", "sql"),
        options("sql-template-strings", "default"),
        options("@other/tags", "unrelated"),
    ] {
        assert_eq!(decisions_for(&source, &heads, &profile), baseline);
    }
}

#[test]
fn direct_tag_heads_need_no_import_ownership_but_local_constants_still_do() {
    let (source, mut ts, _) = fixture("tags.mts");
    ts.module_bindings = None;
    let heads = recollect("tags.mts", &ts);
    let decisions = decisions_for(&source, &heads, &EmbeddedSqlOptions::default());
    for expression in ["read(named`", "read(defaultTag`/*", "read(namespace.sql`"] {
        assert!(decisions.iter().any(|(span, accepted)| {
            *accepted && source[span.0 as usize..].starts_with(expression)
        }));
    }
    assert!(decisions
        .iter()
        .filter(|(span, _)| { source[span.0 as usize..].starts_with("read(tagged)") })
        .all(|(_, accepted)| !accepted));
}

#[test]
fn tag_constants_reject_mutations_and_escapes_but_allow_executor_reads() {
    let (source, _, heads) = fixture("tag-const-effects.mts");
    assert_eq!(
        assert_markers_for(&source, &heads, &options("@bench/tags", "sql")),
        14
    );
}
