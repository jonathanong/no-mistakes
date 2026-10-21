use super::*;

#[test]
fn prepared_profiles_and_root_evaluators_share_one_catalog_without_changing_findings() {
    let mut files = prepared_fixture(
        "helper-tracing-module-ids",
        &["entry", "left/execute", "right/execute"],
    );
    let candidates = files.keys().cloned().collect::<BTreeSet<_>>();
    let resolve = |specifier: &str, from: &std::path::Path| {
        let target = from.parent().unwrap().join(specifier).with_extension("mts");
        candidates.contains(&target).then_some(target)
    };
    let profiles = super::super::super::prepare::profiles(&files, &resolve);
    let mut tables = Vec::new();
    for profile in &profiles {
        for _ in 0..2 {
            let evaluator = super::super::super::evaluator::new(&profile.files, &resolve, false);
            tables.push(Arc::clone(evaluator.shared_module_ids()));
        }
    }
    assert!(tables.iter().all(|ids| Arc::ptr_eq(&tables[0], ids)));
    drop(profiles);
    project(&mut files, resolve);
    for (name, expected) in [
        ("left/execute.mts", "SELECT left_value"),
        ("right/execute.mts", "SELECT right_value"),
    ] {
        let file = files
            .iter()
            .find(|(path, _)| path.ends_with(name))
            .unwrap()
            .1;
        let calls = &file.query_annotation[0].1.calls;
        assert_eq!(calls.len(), 1);
        assert_eq!(calls.values().next().unwrap().as_deref(), Some(expected));
        assert!(crate::codebase::postgres::sql_requires_query_annotation(
            expected
        ));
    }
}
