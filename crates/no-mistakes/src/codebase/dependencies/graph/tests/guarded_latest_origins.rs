use super::*;
use receiver_escapes::{assert_export, assert_unknown, graph};

fn possible(built: &DepGraph, root: &Path, file: &str, callee: &str) {
    let sites = built.call_sites_in_file(&root.join(format!("src/{file}.mts")));
    let site = sites
        .iter()
        .find(|site| site.source_callee == callee)
        .unwrap();
    assert!(site.has_unknown_target(), "{file}: {site:?}");
    let candidates = site.targets().collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1, "{file}: {site:?}");
    assert!(
        matches!(candidates[0].target,
        KnownCallTargetRef::ModuleExport { specifier: "@vendor/valkey", export_path, .. }
        if export_path == callee),
        "{file}: {site:?}"
    );
}

#[test]
fn guarded_latest_origins_keep_alias_method_and_open_tail_candidates() {
    let (built, root) = graph();
    for file in [
        "array-latest-guarded-alias",
        "array-latest-guarded-method",
        "array-latest-guarded-open",
        "array-latest-guarded-push",
        "array-latest-guarded-mutable",
    ] {
        for callee in ["replica.get", "untouched.get"] {
            possible(&built, &root, file, callee);
        }
        assert_export(&built, &root, file, "unrelated.get");
    }
    for callee in ["replica.info", "untouched.info"] {
        possible(&built, &root, "array-latest-guarded-method", callee);
    }
    for file in ["array-latest-guarded-open", "array-latest-guarded-push"] {
        // Moving the guarded literal cannot weaken a guaranteed push or suffix.
        assert_unknown(&built, &root, file, "primary.get");
        assert_export(&built, &root, file, "primary.info");
    }
    for file in ["array-latest-guarded-alias", "array-latest-guarded-method"] {
        assert_export(&built, &root, file, "primary.get");
    }
}

#[test]
fn guarded_latest_origins_do_not_weaken_direct_or_reset_writers() {
    let (built, root) = graph();
    possible(
        &built,
        &root,
        "array-latest-guarded-definite",
        "replica.get",
    );
    assert_unknown(
        &built,
        &root,
        "array-latest-guarded-definite",
        "untouched.get",
    );
    for callee in ["primary.get", "replica.info", "unrelated.get"] {
        assert_export(&built, &root, "array-latest-guarded-definite", callee);
    }
    assert_export(
        &built,
        &root,
        "array-reassign-conditional-reset",
        "replica.get",
    );
    assert_unknown(
        &built,
        &root,
        "array-reassign-conditional-reset",
        "untouched.get",
    );
    // Conditional pushes retain their existing definite projection.
    for file in ["array-push-conditional", "array-push-loop"] {
        for callee in ["replica.get", "untouched.get"] {
            assert_unknown(&built, &root, file, callee);
        }
    }
}

#[test]
fn guarded_latest_common_const_is_definite_before_its_declaration() {
    let (built, root) = graph();
    for file in [
        "array-latest-guarded-deferred-const",
        "array-latest-guarded-deferred-local-const",
    ] {
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
}
