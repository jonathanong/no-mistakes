use super::guarded_latest_origins::possible;
use super::*;
use receiver_escapes::{assert_export, assert_unknown};

#[test]
fn guarded_parameter_epoch_separates_between_and_rhs_assignments() {
    let (built, root) = graph();
    for suffix in ["between", "rhs", "nested", "destructure"] {
        let file = format!("array-latest-guarded-parameter-{suffix}");
        possible(&built, &root, &file, "replica.get");
        if suffix != "destructure" {
            possible(&built, &root, &file, "untouched.get");
        }
        assert_export(&built, &root, &file, "replica.info");
        assert_export(&built, &root, &file, "unrelated.get");
    }
}

#[test]
fn guarded_parameter_epoch_keeps_equal_captures_and_shadowing_definite() {
    let (built, root) = graph();
    for suffix in ["before", "after"] {
        let file = format!("array-latest-guarded-parameter-{suffix}");
        for callee in ["replica.get", "untouched.get"] {
            assert_unknown(&built, &root, &file, callee);
        }
        for callee in ["primary.get", "replica.info", "unrelated.get"] {
            assert_export(&built, &root, &file, callee);
        }
    }
    let file = "array-latest-guarded-parameter-shadow";
    assert_unknown(&built, &root, file, "untouched.get");
    assert_export(&built, &root, file, "replica.get");
    assert_export(&built, &root, file, "unrelated.get");
}

// Keep definite parameter writes separate from unrelated replacement-copy fixtures.
fn graph() -> (DepGraph, std::path::PathBuf) {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("value-flow-guarded-parameters"));
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let built = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    (built, root)
}

#[test]
fn guarded_latest_common_parameter_is_definite() {
    let (built, root) = graph();
    let file = "array-latest-guarded-common-parameter";
    assert_unknown(&built, &root, file, "replica.get");
    for callee in [
        "replica.info",
        "primary.get",
        "untouched.get",
        "unrelated.get",
    ] {
        assert_export(&built, &root, file, callee);
    }
}

#[test]
fn guarded_parameter_epoch_keeps_uncertain_writes_conservative() {
    let (built, root) = graph();
    for suffix in [
        "conditional",
        "foreign",
        "update",
        "arguments",
        "nested-capture",
    ] {
        let file = format!("array-latest-guarded-parameter-{suffix}");
        possible(&built, &root, &file, "replica.get");
        if matches!(suffix, "conditional" | "foreign") {
            possible(&built, &root, &file, "untouched.get");
        }
        for callee in ["primary.get", "replica.info", "unrelated.get"] {
            assert_export(&built, &root, &file, callee);
        }
    }
}
