use super::*;
use receiver_escapes::{assert_export, assert_unknown, graph, graph_for};

pub(super) fn possible(built: &DepGraph, root: &Path, file: &str, callee: &str) {
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
        // The pushed tail is definite; an unknown prefix can shift the ordinary suffix.
        if file == "array-latest-guarded-open" {
            possible(&built, &root, file, "primary.get");
        } else {
            assert_unknown(&built, &root, file, "primary.get");
        }
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
    // A valid no-push execution leaves replica's original method callable.
    for file in ["array-push-conditional", "array-push-loop"] {
        possible(&built, &root, file, "replica.get");
        assert_unknown(&built, &root, file, "untouched.get");
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

#[test]
fn guarded_latest_common_push_is_definite() {
    let (built, root) = graph();
    let file = "array-latest-guarded-common-push";
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
fn guarded_push_common_proof_keeps_optional_tails_possible() {
    let (built, root) = graph();
    for file in [
        "array-latest-guarded-push-tail",
        "array-latest-guarded-conditional-push-prefix",
    ] {
        assert_unknown(&built, &root, file, "replica.get");
        possible(&built, &root, file, "untouched.get");
        assert_export(&built, &root, file, "replica.info");
        assert_export(&built, &root, file, "unrelated.get");
    }
    possible(
        &built,
        &root,
        "array-latest-guarded-push-tail",
        "primary.get",
    );
    assert_export(
        &built,
        &root,
        "array-latest-guarded-conditional-push-prefix",
        "primary.get",
    );
    for callee in ["replica.get", "untouched.get"] {
        possible(&built, &root, "array-latest-guarded-varying-push", callee);
    }
    assert_export(
        &built,
        &root,
        "array-latest-guarded-varying-push",
        "primary.get",
    );
}

#[test]
fn guarded_push_common_proof_requires_certain_execution_and_width() {
    let (built, root) = graph();
    let file = "array-latest-guarded-uncertain-push";
    for callee in ["replica.get", "untouched.info", "primary.del"] {
        possible(&built, &root, file, callee);
    }
    assert_export(&built, &root, file, "unrelated.get");
}

#[test]
fn guarded_push_proof_excludes_replacement_initializer_calls() {
    let (built, root) = graph();
    let file = "array-latest-guarded-push-initializer";
    possible(&built, &root, file, "replica.get");
    for callee in ["primary.get", "replica.info", "unrelated.get"] {
        assert_export(&built, &root, file, callee);
    }
}

#[test]
fn known_empty_prefix_cannot_send_the_suffix_to_the_first_parameter() {
    let (built, root) = graph();
    let file = "array-latest-guarded-empty-prefix";
    for callee in ["replica.get", "untouched.get"] {
        possible(&built, &root, file, callee);
    }
    for callee in ["primary.get", "primary.info", "unrelated.get"] {
        assert_export(&built, &root, file, callee);
    }
}

#[test]
fn literal_suffix_after_an_open_guarded_spread_remains_possible() {
    let (built, root) = graph();
    let file = "array-latest-guarded-open-literal";
    for callee in ["replica.get", "untouched.get", "primary.get"] {
        possible(&built, &root, file, callee);
    }
    for callee in ["primary.info", "unrelated.get"] {
        assert_export(&built, &root, file, callee);
    }
}

#[test]
fn unconditional_parameter_rebinding_does_not_write_the_original_argument() {
    let (built, root) = graph();
    let file = "reassigned-parameter";
    assert_unknown(&built, &root, file, "replica.get");
    assert_export(&built, &root, file, "untouched.get");
    assert_export(&built, &root, file, "primary.get");
}

#[test]
fn optional_pushes_keep_later_pushes_and_suffixes_possible() {
    let (built, root) = graph();
    for file in [
        "array-optional-push-then-push",
        "array-optional-push-then-suffix",
        "array-optional-push-forwarded",
        "array-optional-push-function-call",
        "array-optional-push-method-call",
    ] {
        for callee in ["replica.get", "untouched.get"] {
            possible(&built, &root, file, callee);
        }
        for callee in ["primary.get", "replica.info", "unrelated.get"] {
            assert_export(&built, &root, file, callee);
        }
    }
}

#[test]
fn fixed_arguments_before_the_optional_spread_keep_their_write_origins() {
    let (built, root) = graph_for("value-flow-optional-prefixes");
    for file in [
        "array-optional-push-after-fixed-prefix",
        "array-optional-push-after-fixed-prefix-call",
        "array-optional-push-after-fixed-prefix-forwarded",
    ] {
        // Both runtime flag values overwrite this vendor method locally.
        assert_unknown(&built, &root, file, "replica.get");
        for callee in [
            "untouched.get",
            "primary.get",
            "replica.info",
            "unrelated.get",
        ] {
            assert_export(&built, &root, file, callee);
        }
    }
}

#[test]
fn empty_and_wide_prefixes_preserve_actual_argument_positions() {
    let (built, root) = graph_for("value-flow-optional-prefixes");
    for file in [
        "array-optional-push-after-empty-prefix",
        "array-optional-push-after-wide-prefix",
    ] {
        for callee in [
            "replica.get",
            "primary.get",
            "replica.info",
            "unrelated.get",
        ] {
            assert_export(&built, &root, file, callee);
        }
    }
    possible(
        &built,
        &root,
        "array-optional-push-after-empty-prefix",
        "untouched.get",
    );
    assert_export(
        &built,
        &root,
        "array-optional-push-after-wide-prefix",
        "untouched.get",
    );
}

#[test]
fn deferred_pushes_keep_shifted_suffixes_and_receiver_candidates() {
    let (built, root) = graph_for("value-flow-optional-prefixes");
    let file = "array-optional-push-conditional-helper";
    for callee in ["replica.get", "untouched.get"] {
        possible(&built, &root, file, callee);
    }
    let file = "array-optional-push-uncalled-helper";
    // The projection does not prove helper reachability: keep the untouched
    // original receiver while still recording that the suffix may be written.
    possible(&built, &root, file, "replica.get");
    let sites = built.call_sites_in_file(&root.join(format!("src/{file}.mts")));
    assert!(sites
        .iter()
        .find(|site| site.source_callee == "untouched.get")
        .unwrap()
        .has_unknown_target());
    for file in [
        "array-optional-push-uncalled-helper",
        "array-optional-push-conditional-helper",
    ] {
        for callee in ["primary.get", "replica.info", "unrelated.get"] {
            assert_export(&built, &root, file, callee);
        }
    }
}
