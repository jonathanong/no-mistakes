use std::collections::BTreeSet;

fn failures(scenario: &str, source: &[&str]) -> Vec<String> {
    let root = super::repo_root()
        .join("fixtures/docs-rule-inventory")
        .join(scenario);
    let source: BTreeSet<_> = source.iter().map(|name| (*name).to_owned()).collect();
    super::rule_inventory::inventory_failures(&root, &source)
}

fn assert_failures(scenario: &str, source: &[&str], expected: &[&str]) {
    let mut actual = failures(scenario, source);
    let mut expected: Vec<_> = expected
        .iter()
        .map(|message| (*message).to_owned())
        .collect();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected, "{scenario}");
}

#[test]
fn adding_a_source_rule_with_its_page_and_index_needs_no_count_update() {
    assert_failures("one-rule", &["alpha-rule"], &[]);
    assert_failures("two-rules", &["alpha-rule", "beta-rule"], &[]);
}

#[test]
fn newly_added_rule_without_documentation_names_both_missing_surfaces() {
    assert_failures(
        "added-without-docs",
        &["alpha-rule", "beta-rule"],
        &[
            "missing rule page: beta-rule",
            "missing README entry: beta-rule",
        ],
    );
}

#[test]
fn deleting_a_page_fails_even_when_its_readme_link_survives() {
    assert_failures(
        "deleted-page",
        &["alpha-rule", "beta-rule"],
        &["missing rule page: beta-rule"],
    );
}

#[test]
fn orphan_pages_are_reported_for_canonical_and_prose_headings() {
    for (scenario, expected) in [
        ("orphan-canonical", "extra rule page: stale-rule"),
        ("orphan-prose", "extra rule page: orphan-guide"),
    ] {
        assert_failures(scenario, &["alpha-rule"], &[expected]);
    }
}

#[test]
fn filenames_and_code_samples_do_not_count_as_readme_index_entries() {
    for scenario in ["readme-prose-only", "readme-code-only"] {
        assert_failures(
            scenario,
            &["alpha-rule", "beta-rule"],
            &["missing README entry: beta-rule"],
        );
    }
}

#[test]
fn an_extra_readme_link_is_reported_without_needing_an_actual_page() {
    assert_failures(
        "extra-readme-entry",
        &["alpha-rule"],
        &["extra README entry: stale-rule"],
    );
}

#[test]
fn reference_links_and_anchor_fragments_are_valid_index_entries() {
    assert_failures("reference-anchors", &["alpha-rule", "beta-rule"], &[]);
}

#[test]
fn source_owned_guides_follow_link_chains_and_cycles_without_a_name_allowlist() {
    // A fenced canonical-looking heading is prose, not a stale rule page.
    assert_failures("guide-chain-cycle", &["alpha-rule"], &[]);
}

#[test]
fn a_linked_canonical_rule_cannot_hide_as_a_supporting_guide() {
    assert_failures(
        "linked-stale-canonical",
        &["alpha-rule"],
        &["extra rule page: stale-rule"],
    );
}

#[test]
fn readme_links_do_not_grant_guide_ownership() {
    assert_failures(
        "readme-owned-orphan",
        &["alpha-rule"],
        &[
            "extra rule page: unowned-guide",
            "extra README entry: unowned-guide",
        ],
    );
}

#[test]
fn external_parent_directory_and_non_markdown_links_are_not_rule_entries() {
    assert_failures("ignored-readme-links", &["alpha-rule"], &[]);
}
