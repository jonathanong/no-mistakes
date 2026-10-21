use super::super::super::{Evaluator, File};
use crate::codebase::postgres::query_annotation;
use crate::codebase::postgres::query_annotation::evaluate::Value;
use crate::codebase::ts_source::FileIdMap;
use std::collections::BTreeSet;

#[path = "private_arms/support.rs"]
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) mod support;
use support::fixture;

fn private_ids<F: Fn(&str, &std::path::Path) -> Option<std::path::PathBuf>>(
    evaluator: &Evaluator<'_, F>,
    value: &Value,
) -> Vec<(u32, usize, u64)> {
    support::functions(value)
        .into_iter()
        .map(|(start, env)| {
            let owner = evaluator
                .captured_bindings
                .get(&env)
                .and_then(|origins| origins.get("slots"))
                .copied()
                .unwrap_or(env);
            let Value::Arguments(id) = evaluator.scopes[owner]["slots"] else {
                panic!("the saved closure must retain its private argument object");
            };
            (start, env, id)
        })
        .collect()
}

#[test]
fn same_factory_sibling_callbacks_keep_distinct_frames_and_sql_candidates() {
    let (path, source, file) = fixture("no-deletion");
    let files = support::files(&path, &file.query_annotation[0].1, &file.embedded_sql[0].1);
    let mut evaluator = support::evaluator(&files);
    let root = evaluator.module_environment(&path);
    let ids = private_ids(&evaluator, &evaluator.scopes[root]["selected"]);
    assert_eq!(ids.len(), 2);
    assert_eq!(
        ids[0].0, ids[1].0,
        "one factory closure syntax, not two different callbacks"
    );
    assert_ne!(ids[0].1, ids[1].1, "private captured frames must not alias");
    assert_ne!(
        ids[0].2, ids[1].2,
        "private argument identities must not alias"
    );
    let start = source.find("write(slots[0])").unwrap() as u32;
    let events = &evaluator.events[&(
        path.clone(),
        (start, start + "write(slots[0])".len() as u32),
    )];
    assert_eq!(
        events.len(),
        2,
        "deduplication must not collapse private callbacks"
    );
    let sql = events
        .iter()
        .map(|(_, value)| match value.clone().exposed() {
            Value::Possible(values) => {
                assert_eq!(values.len(), 1);
                let Value::Prefix(text, true, _) = values[0].clone().exposed() else {
                    panic!("each opaque callback must retain its concrete SQL candidate");
                };
                text
            }
            _ => panic!("opaque consumers must retain uncertainty about argument effects"),
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        sql,
        BTreeSet::from(["SELECT 101".into(), "/* safe */ SELECT 202".into()])
    );
    drop(evaluator);
    drop(files);
    let mut prepared = FileIdMap::from_iter([(path.clone(), file)]);
    let projected = query_annotation::project::project_executed(&mut prepared, |_, _| None);
    // The opaque consumer may mutate arguments; known candidates are not a
    // complete SQL projection. The unsafe candidate still produces a finding.
    assert_eq!(
        projected[&(
            path.clone(),
            (start, start + "write(slots[0])".len() as u32),
        )],
        Err("unresolved-value-or-effect")
    );
    assert_eq!(
        prepared[&path].query_annotation[0]
            .1
            .calls
            .values()
            .filter_map(Option::as_deref)
            .collect::<Vec<_>>(),
        ["SELECT 101"]
    );
}

#[test]
fn pristine_independent_arms_reuse_numbers_and_therefore_require_translation() {
    let (path, _, file) = fixture("no-deletion");
    let mut facts = file.query_annotation[0].1.clone();
    let query_annotation::Expr::Sequence(parts) = facts.globals["selected"].clone() else {
        panic!("saved condition must retain its expression effects");
    };
    let query_annotation::Expr::Alternatives(arms) = parts.last().unwrap().clone() else {
        panic!("saved unknown condition must retain both factory calls");
    };
    // Keep the saved factory AST and setup, stopping before the branch itself.
    let boundary = facts
        .roots
        .iter()
        .position(|step| {
            matches!(step,
        query_annotation::Step::Bind(name, _) if name == "selected")
        })
        .unwrap();
    facts.roots.truncate(boundary);
    let files = support::files(&path, &facts, &file.embedded_sql[0].1);
    let ids = arms
        .iter()
        .map(|arm| {
            let mut evaluator = support::evaluator(&files);
            let root = evaluator.module_environment(&path);
            let returned = evaluator.expr(arm, &path, &root, 16, false);
            let ids = private_ids(&evaluator, &returned);
            assert_eq!(ids.len(), 1);
            let sql = evaluator.argument_objects[&ids[0].2][0].clone().exposed();
            let Value::Prefix(sql, true, _) = sql else {
                panic!("saved factory argument")
            };
            (ids[0], sql)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ids[0].0, ids[1].0,
        "fresh arms collide in both numeric namespaces"
    );
    assert_ne!(
        ids[0].1, ids[1].1,
        "the colliding objects hold different SQL"
    );
}

#[test]
fn private_deletion_never_hides_the_other_siblings_unsafe_query() {
    let (path, source, file) = fixture("deletion");
    let files = support::files(&path, &file.query_annotation[0].1, &file.embedded_sql[0].1);
    let mut evaluator = support::evaluator(&files);
    let root = evaluator.module_environment(&path);
    let ids = private_ids(&evaluator, &evaluator.scopes[root]["selected"]);
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0].2, ids[1].2);
    let deleted = ids
        .iter()
        .filter(|(_, _, id)| {
            evaluator
                .definite_deleted_argument_slots
                .contains(&(*id, 0))
        })
        .count();
    assert_eq!(
        deleted, 1,
        "only the deleting sibling owns this definite mask"
    );
    let start = source.find("write(slots[0])").unwrap() as u32;
    let events = &evaluator.events[&(
        path.clone(),
        (start, start + "write(slots[0])".len() as u32),
    )];
    assert!(events.iter().any(|(_, value)| {
        let Value::Possible(values) = value.clone().exposed() else {
            return false;
        };
        matches!(values.as_slice(), [value] if matches!(value.clone().exposed(), Value::Prefix(text, true, _) if text == "SELECT 101"))
    }));
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn saved_private_callbacks_keep_exact_findings_with_diagnostics_off_and_on() {
    support::assert_report();
}
