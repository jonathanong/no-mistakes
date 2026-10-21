use super::cli_fetches::{fetches_cmd, fixture};

#[test]
fn test_cli_target_duplicates_are_deduplicated() {
    let root = fixture("target-duplicates-are-deduplicated");
    let page = root.join("app/page.tsx");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root).arg(&page).arg(&page);
    cmd.assert().success();
}

#[test]
fn test_cli_target_file_match_uses_wrapper_chain() {
    let root = fixture("target-file-match-uses-wrapper-chain");
    let layout = root.join("app/layout.tsx");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root).arg(&layout);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/layout"));
}

#[test]
fn test_cli_target_file_match_uses_wrapper_chain_via_import() {
    let root = fixture("target-file-match-uses-wrapper-chain-via-import");
    let target = root.join("app/dashboard/target.ts");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root).arg(&target);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/layout"));
}

#[test]
fn test_cli_target_file_match_skips_non_matching_wrapper_then_matches_outer() {
    // The inner layout deliberately omits the target; only the outer layout imports it.
    let root = fixture("target-file-match-skips-non-matching-wrapper-then-matches-outer");
    let target = root.join("app/target.ts");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root).arg(&target);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/api/root"));
}

#[test]
fn test_cli_includes_client_side_fetches() {
    let root = fixture("includes-client-side-fetches");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root);
    cmd.assert().success().stdout(predicates::str::contains(
        "| GET | `/api/client` | client |",
    ));
}

#[test]
fn test_cli_sorts_multiple_unsupported_fetches() {
    let root = fixture("sorts-multiple-unsupported-fetches");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("## Unsupported (Dynamic)"))
        .stdout(predicates::str::contains("### / (app/page.tsx)"))
        .stdout(predicates::str::contains("### /about (app/about/page.tsx)"));
}

#[test]
fn test_cli_includes_duplicates_and_unsupported_sections() {
    let root = fixture("includes-duplicates-and-unsupported-sections");

    let mut cmd = fetches_cmd();
    cmd.arg("--root").arg(&root);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("## Duplicates"))
        .stdout(predicates::str::contains("## Unsupported (Dynamic)"));
}
