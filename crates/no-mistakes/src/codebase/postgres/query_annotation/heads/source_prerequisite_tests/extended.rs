use super::*;

#[test]
fn returned_conditional_requires_every_arm_and_keeps_failed_source_demand() {
    let (mut files, root) = prepared();
    let path = root.join("returned-conditional.mts");
    let heads = files[&path].query_annotation_heads.as_ref().unwrap();
    assert_eq!(heads.pending_returned_params.len(), 1);
    assert_eq!(heads.pending_returned_params[0].imports.len(), 2);
    let span = heads.pending_returned_params[0].span;
    assert!(files[&root.join("literal-helper.mts")]
        .query_annotation_heads
        .as_ref()
        .unwrap()
        .exported_returned_params
        .contains("suffix"));
    let resolver = resolve(&files);
    super::super::super::project(&mut files, resolver);
    let file = &files[&path];
    assert_eq!(
        file.query_annotation_heads.as_ref().unwrap().calls[&span],
        super::super::super::HeadProof::Unproven
    );
    assert_eq!(
        failures(file, 0),
        [root.join("broken.mts")].into_iter().collect()
    );
    assert!(failures(file, 1).is_empty());
}

#[test]
fn unresolved_returned_conditional_imports_abstain_without_source_failures() {
    let (mut files, root) = prepared();
    let path = root.join("returned-conditional.mts");
    let span = files[&path]
        .query_annotation_heads
        .as_ref()
        .unwrap()
        .pending_returned_params[0]
        .span;
    super::super::super::project(&mut files, |_, _| None);
    let file = &files[&path];
    assert_eq!(
        file.query_annotation_heads.as_ref().unwrap().calls[&span],
        super::super::super::HeadProof::Unproven
    );
    assert!(failures(file, 0).is_empty());
}

#[test]
fn returned_builder_and_async_append_effect_failures_reach_executor_demands() {
    let (mut files, root) = prepared();
    let returned = &files[&root.join("returned.mts")];
    assert_eq!(
        returned
            .query_annotation_heads
            .as_ref()
            .unwrap()
            .pending_returned_params
            .len(),
        1
    );
    let factory = &files[&root.join("async-factory.mts")];
    assert!(factory
        .query_annotation_heads
        .as_ref()
        .unwrap()
        .exported_async_returns
        .contains_key("factory"));
    let caller = files[&root.join("async-effects.mts")]
        .query_annotation_heads
        .as_ref()
        .unwrap();
    assert_eq!(caller.pending_async_returns.len(), 1);
    let resolver = resolve(&files);
    super::super::super::project(&mut files, resolver);
    let expected = BTreeSet::from([root.join("broken.mts")]);
    for name in ["returned", "async-effects"] {
        let file = &files[&root.join(format!("{name}.mts"))];
        assert_eq!(failures(file, 0), expected, "{name}");
        assert!(failures(file, 1).is_empty(), "{name}");
    }
}

#[test]
fn absent_prepared_targets_remain_unproven_without_claiming_a_source_failure() {
    for missing in ["broken.mts", "async-factory.mts"] {
        let (mut files, root) = prepared();
        let target = root.join(missing);
        let resolver = resolve(&files);
        files.remove(&target);
        super::super::super::project(&mut files, resolver);
        let owners: &[&str] = if missing == "broken.mts" {
            &[
                "direct",
                "returned",
                "async",
                "async-effects",
                "callback",
                "effect",
            ]
        } else {
            &["async-effects"]
        };
        for name in owners {
            let file = &files[&root.join(format!("{name}.mts"))];
            assert!(failures(file, 0).is_empty(), "{missing}: {name}");
        }
    }
}
