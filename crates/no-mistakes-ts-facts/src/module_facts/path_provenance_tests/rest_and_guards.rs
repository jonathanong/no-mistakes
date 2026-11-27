use super::*;

fn marked_offset(read: &Read, case: &str) -> u32 {
    let line = read
        .source
        .lines()
        .find(|line| line.contains(&format!("'{case}'")))
        .unwrap();
    let prefix = line.find("readdirSync(").unwrap();
    (read.source.find(line).unwrap() + prefix) as u32
}

fn marked_origin(read: &Read, case: &str) -> PathOrigin {
    read.facts.call_arguments.as_ref().unwrap()[&marked_offset(read, case)]
        .path
        .clone()
        .unwrap()
}

#[test]
fn object_rest_preserves_unexcluded_parameter_members_and_default_uncertainty() {
    let read = read("3304-object-rest.mts");
    for case in [
        "parameter-member",
        "empty-default",
        "populated-default",
        "local-parameter",
    ] {
        let origin = marked_origin(&read, case);
        assert_eq!(origin.param, Some(0), "{case}: {origin:?}");
        assert_eq!(
            origin.param_member.as_deref(),
            Some("root"),
            "{case}: {origin:?}"
        );
        assert!(origin.param_owner.is_some(), "{case}: {origin:?}");
        assert_eq!(
            origin.param_default.is_some(),
            case == "populated-default",
            "{case}: {origin:?}"
        );
    }
    let local = marked_origin(&read, "local-object");
    assert_eq!(local.anchor, PathAnchor::Relative, "{local:?}");
    assert_eq!(local.segments, ["fixtures"], "{local:?}");
    for case in [
        "excluded-member",
        "computed-pattern",
        "local-excluded",
        "source-reassigned",
        "rest-escaped",
        "rest-written",
        "renamed-exclusion",
        "nested-pattern",
    ] {
        let origin = marked_origin(&read, case);
        assert_eq!(origin, PathOrigin::unknown(), "{case}: {origin:?}");
    }
    for case in ["spread-excluded", "spread-parameter"] {
        let members = &read.provenance().arg_member_paths[&marked_offset(&read, case)];
        let (_, _, origin) = members
            .iter()
            .find(|(index, key, _)| *index == 0 && key == "root")
            .unwrap_or_else(|| panic!("{case}: {members:?}"));
        assert_eq!(origin.anchor, PathAnchor::Relative, "{case}: {origin:?}");
        assert_eq!(origin.segments, ["fixtures"], "{case}: {origin:?}");
    }
    for case in ["spread-present", "spread-computed", "spread-unsafe"] {
        assert!(
            read.provenance()
                .arg_member_paths
                .get(&marked_offset(&read, case))
                .is_none_or(|members| members
                    .iter()
                    .all(|(index, key, _)| *index != 0 || key != "root")),
            "{case}: {:?}",
            read.provenance().arg_member_paths
        );
    }
}

#[test]
fn array_guards_record_only_first_plain_immutable_destructuring_with_strict_tests() {
    let read = read("3304-array-guards.mts");
    for (case, expected) in [
        ("direct", vec![(0, false, serde_json::json!("scan"))]),
        ("reversed", vec![(0, false, serde_json::json!("scan"))]),
        ("hole", vec![(1, false, serde_json::json!("scan"))]),
        ("numeric", vec![(0, false, serde_json::json!(2.0))]),
        (
            "conjunction",
            vec![
                (0, false, serde_json::json!("scan")),
                (1, true, serde_json::json!(1.0)),
            ],
        ),
        ("rest-forwarded", vec![(1, true, serde_json::json!(1.0))]),
    ] {
        let guards = &read.provenance().call_guards[&marked_offset(&read, case)];
        let actual: Vec<_> = guards
            .iter()
            .map(|guard| (guard.index, guard.length, guard.expected.clone()))
            .collect();
        assert_eq!(actual, expected, "{case}: {guards:?}");
        assert!(guards
            .iter()
            .all(|guard| guard.parameter == 0 && guard.owner < marked_offset(&read, case)));
    }
    for case in [
        "loose",
        "disjunction",
        "alternate",
        "late",
        "multiple",
        "defaulted",
        "extra",
        "reused",
        "nested",
        "written",
        "rest-escaped",
        "rest-wrong-comparison",
        "direct-member",
        "boolean",
    ] {
        assert!(
            read.provenance()
                .call_guards
                .get(&marked_offset(&read, case))
                .is_none_or(Vec::is_empty),
            "{case}: {:?}",
            read.provenance().call_guards
        );
    }
}
