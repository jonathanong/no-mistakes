use super::{
    deadline_checked_paths, discover_files, discover_files_from_visible, discover_source_files,
    discover_source_files_from_visible, discover_visible_paths, format_parse_diagnostic,
    git_visible_files, has_disable_comment, has_disable_file_comment, has_disable_line_comment,
    is_skipped_dir, is_test_file, line_number, matching_disable_directive,
    normalize_discovery_path, parse_git_tagged_paths, relative_slash_path, starts_with_use_client,
    static_property_key_name, unwrap_ts_wrappers, walk_files, FrozenPathRemapper,
};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectPropertyKind, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

mod disable_comments;
mod discovery_preserve;
mod gitignore;
mod ignore_walk;
mod source_and_discovery;
mod visible_snapshot_scopes;

#[test]
fn matching_disable_directive_reports_exact_supported_provenance() {
    assert_eq!(
        matching_disable_directive(
            "/* notice */\n// no-mistakes-disable-file my-rule\nconst value = 1",
            Some(3),
            "my-rule",
        ),
        Some(super::DisableDirective::File { line: 2 })
    );
    assert_eq!(
        matching_disable_directive(
            "value(); // no-mistakes-disable-line my-rule",
            Some(1),
            "my-rule"
        ),
        Some(super::DisableDirective::Line { line: 1 })
    );
    assert_eq!(
        matching_disable_directive(
            "// no-mistakes-disable-next-line my-rule\nvalue()",
            Some(2),
            "my-rule"
        ),
        Some(super::DisableDirective::NextLine { line: 1 })
    );
}

#[test]
fn frozen_path_remapper_keeps_symlink_namespace_deterministic() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/symlink-workspace");
    let lexical = root.join("link/src/value.ts");
    let real = root.join("real/src/value.ts");
    let normalized_real = root.join("real/../real/src/value.ts");
    let noncanonical_link = root.join("link/../link/src/value.ts");
    let missing_member = PathBuf::from("/definitely/missing-member.ts");
    let missing = PathBuf::from("/definitely/missing.ts");
    let first = FrozenPathRemapper::from_paths(vec![real.clone(), lexical.clone(), missing_member]);
    let reversed = FrozenPathRemapper::from_paths(vec![lexical.clone(), real.clone()]);

    // Exact lexical paths retain their own graph identities.
    assert_eq!(first.remap(&lexical), Some(lexical.clone()));
    // A normalized alias is ambiguous because both lexical paths refer to the
    // same file, so it must not select one based on discovery order.
    assert_eq!(first.remap(&normalized_real), None);
    assert_eq!(first.remap(&noncanonical_link), None);
    assert_eq!(reversed.remap(&noncanonical_link), None);
    assert_eq!(first.remap(&missing), None);
}

#[test]
fn frozen_path_remapper_rejects_ambiguous_canonical_membership() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/symlink-workspace");
    let lexical = root.join("link/src/value.ts");
    let real = root.join("real/src/value.ts");
    let remapper = FrozenPathRemapper::from_paths(vec![lexical, real]);

    assert_eq!(
        remapper.remap(&root.join("link/../link/src/value.ts")),
        None,
        "a non-exact lookup must not arbitrarily select one of two lexical paths to the same file"
    );
}

#[test]
fn frozen_path_remapper_preserves_exact_links_without_canonical_fallback() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/symlink-workspace");
    let lexical = root.join("link/src/value.ts");
    let remapper = FrozenPathRemapper::from_paths(vec![lexical.clone()]);

    assert_eq!(remapper.remap(&lexical), Some(lexical));
    assert_eq!(
        remapper.remap(&root.join("link/src/value.ts")),
        Some(root.join("link/src/value.ts"))
    );

    let real = root.join("real/src/value.ts");
    let remapper = FrozenPathRemapper::from_paths(vec![real.clone()]);
    assert_eq!(
        remapper.remap(&root.join("real/../real/src/value.ts")),
        Some(real),
        "a normalized direct lookup retains the only visible spelling"
    );
}

#[test]
fn frozen_path_remapper_follows_actual_filesystem_case_resolution() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/symlink-workspace");
    let tracked = root.join("real/src/value.ts");
    let remapper = FrozenPathRemapper::from_paths(vec![tracked.clone()]);

    let candidate = root.join("real/src/VALUE.ts");
    let case_variant_resolves = candidate.canonicalize().ok() == tracked.canonicalize().ok();
    assert_eq!(
        remapper.remap(&candidate),
        case_variant_resolves.then_some(tracked)
    );
}

#[test]
fn frozen_path_remapper_rejects_case_variant_symlink_real_ambiguity() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/symlink-workspace");
    let remapper = FrozenPathRemapper::from_paths(vec![
        root.join("link/src/value.ts"),
        root.join("real/src/value.ts"),
    ]);

    let candidate = root.join("LINK/src/VALUE.ts");
    let case_variant_reaches_the_file = candidate.canonicalize().is_ok();
    assert_eq!(remapper.remap(&candidate), None);
    if case_variant_reaches_the_file {
        assert!(
            remapper.remap(&candidate).is_none(),
            "case-insensitive aliases still reject the symlink/real ambiguity"
        );
    }
}

#[test]
fn deadline_checked_paths_preserves_success_and_maps_timeout() {
    assert!(deadline_checked_paths(Vec::new(), Ok(()))
        .unwrap()
        .is_empty());

    let error = deadline_checked_paths(Vec::new(), Err(anyhow::anyhow!("expired"))).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert_eq!(error.to_string(), "expired");
}

#[test]
fn parse_diagnostic_formatter_preserves_diagnostic_and_panic_messages() {
    let path = Path::new("/repo/bad.ts");
    let diagnostics = [oxc_diagnostics::OxcDiagnostic::error("bad syntax")];

    assert_eq!(
        format_parse_diagnostic(path, &diagnostics),
        format!("parsing {}: {:?}", path.display(), diagnostics[0])
    );
    assert_eq!(
        format_parse_diagnostic(path, &[]),
        "parsing /repo/bad.ts: parser panicked without diagnostic details"
    );
}

fn git_init(dir: &Path) {
    crate::test_support::git_init(dir);
}

fn git_add_all(dir: &Path) {
    crate::test_support::git_add_all(dir);
}

fn saved_discovery_fixture(name: &str) -> TempDir {
    let source = crate::test_support::fixture_root("ts-source").join(name);
    crate::test_support::materialize_saved_fixture(&source)
}

fn fixture(path: &str) -> PathBuf {
    let mut parts = path.splitn(3, '/');
    let category = parts.next().unwrap_or(path);
    let sub = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("");
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(category)
        .join(sub)
        .join("fixture");
    if !rest.is_empty() {
        p = p.join(rest);
    }
    p
}
