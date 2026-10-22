use super::support::{assert_markers, decisions, fixture, recollect};
#[test]
fn any_tag_uses_the_first_quasi_without_import_or_profile_trust() {
    let (source, _, heads) = fixture("tags.mts");
    assert_eq!(assert_markers(&source, &heads), 35);
}

#[test]
fn direct_tag_heads_need_no_import_ownership_but_local_constants_still_do() {
    let (source, mut ts, _) = fixture("tags.mts");
    ts.module_bindings = None;
    let heads = recollect("tags.mts", &ts);
    let decisions = decisions(&source, &heads);
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
    assert_eq!(assert_markers(&source, &heads), 17);
}
