//! Regression coverage for
//! <https://github.com/jonathanong/no-mistakes/issues/624> (findings dropped
//! or misattributed across multiple `type: nextjs` projects) and
//! <https://github.com/jonathanong/no-mistakes/issues/625> (`src/app` route
//! patterns coming out as `/src/app/...` instead of `/...`), using the
//! shared fixture at `fixtures/playwright/multi-frontend-apps/`.

use super::super::*;
use crate::codebase::rules::RuleFinding;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn fixture_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/playwright/multi-frontend-apps")
}

/// Materializes the shared fixture into a fresh temp directory (so
/// `discover_visible_paths` falls back to the ignore-based walker instead of
/// treating it as part of this repository's own git tree) and loads the
/// named `.no-mistakes.yml` variant explicitly.
fn load(config_name: &str) -> (tempfile::TempDir, PathBuf, NoMistakesConfig) {
    let fixture = crate::test_support::materialize_saved_fixture(&fixture_source());
    let root = fixture.path().canonicalize().unwrap();
    let config_path = root.join(config_name);
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    (fixture, config_path, config)
}

fn check_scenario(config_name: &str) -> Result<Vec<RuleFinding>> {
    let (fixture, config_path, config) = load(config_name);
    let result = check(
        fixture.path().canonicalize().unwrap().as_path(),
        Some(&config_path),
        &config,
    );
    drop(fixture);
    result
}

/// #624's exact repro shape: two `type: nextjs` projects, each Playwright
/// project scoped only via `tests.playwright`, no `projects:` or `apps:`
/// binding. This must now fail loudly instead of silently picking whichever
/// project sorts first.
#[test]
fn unbound_config_is_an_ambiguity_error() {
    let error = check_scenario("unbound.no-mistakes.yml").unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("agent-web"), "{message}");
    assert!(message.contains("control-web"), "{message}");
    assert!(message.contains("tests.playwright.apps"), "{message}");
}

/// The core #624 fix: binding via `rules[].projects` makes the combined run
/// exactly the union of what each app produces in isolation — not one
/// project's findings, and not some other count in between.
#[test]
fn bound_config_is_additive_across_isolated_runs() {
    let combined = check_scenario("bound.no-mistakes.yml").unwrap();
    let control_only = check_scenario("control-only.no-mistakes.yml").unwrap();
    let agent_only = check_scenario("agent-only.no-mistakes.yml").unwrap();

    assert!(!control_only.is_empty(), "control-only run found nothing");
    assert!(!agent_only.is_empty(), "agent-only run found nothing");

    let expected: BTreeSet<RuleFinding> = control_only
        .iter()
        .chain(agent_only.iter())
        .cloned()
        .collect();
    let actual: BTreeSet<RuleFinding> = combined.into_iter().collect();
    assert_eq!(actual, expected);

    // Each app's own gap is present, and only under that app's own path —
    // proving the two scoped analyses didn't cross-contaminate.
    assert!(control_only
        .iter()
        .any(|f| f.file.contains("control-web") && f.message.contains("/hosts")));
    assert!(agent_only
        .iter()
        .any(|f| f.file.contains("agent-web") && f.message.contains("/tasks")));
    assert!(!control_only.iter().any(|f| f.file.contains("agent-web")));
    assert!(!agent_only.iter().any(|f| f.file.contains("control-web")));
}

/// #625: route patterns are `/hosts`, not `/src/app/hosts`, once the app's
/// route root is resolved through `frontend_apps`'s `src/app`-preferred
/// probe.
#[test]
fn route_patterns_have_no_src_app_prefix() {
    let control_only = check_scenario("control-only.no-mistakes.yml").unwrap();
    let uncovered = control_only
        .iter()
        .find(|f| f.rule == PLAYWRIGHT_COVERAGE && f.message.contains("not covered"))
        .expect("expected an uncovered-route finding");
    assert!(
        uncovered.message.contains("`/hosts`"),
        "{}",
        uncovered.message
    );
    assert!(
        !uncovered.message.contains("src/app"),
        "{}",
        uncovered.message
    );
}

/// The alternate binding mechanism (`tests.playwright.apps.<project>.project`
/// instead of `rules[].projects`) resolves to the identical result.
#[test]
fn app_override_config_matches_rules_projects_binding() {
    let bound = check_scenario("bound.no-mistakes.yml").unwrap();
    let overridden = check_scenario("app-override.no-mistakes.yml").unwrap();
    assert_eq!(
        bound.into_iter().collect::<BTreeSet<_>>(),
        overridden.into_iter().collect::<BTreeSet<_>>()
    );
}

/// The duplicate `data-pw="shared-cta"` value in both apps is invisible to
/// `playwright-unique-test-ids` when the rules are app-scoped (each scan only
/// ever sees its own app's single occurrence)...
#[test]
fn duplicate_test_id_is_not_flagged_when_app_scoped() {
    let findings = check_scenario("bound.no-mistakes.yml").unwrap();
    assert!(
        !findings
            .iter()
            .any(|f| f.rule == PLAYWRIGHT_UNIQUE_TEST_IDS),
        "{findings:?}"
    );
}

/// ...but is correctly flagged when a rule intentionally spans both apps via
/// explicit `frontendRoot`/`selectorRoots` overrides.
#[test]
fn duplicate_test_id_is_flagged_when_scan_spans_both_apps() {
    let findings = check_scenario("combined.no-mistakes.yml").unwrap();
    let duplicate = findings
        .iter()
        .find(|f| f.rule == PLAYWRIGHT_UNIQUE_TEST_IDS)
        .expect("expected a duplicate-test-id finding");
    assert!(
        duplicate.message.contains("shared-cta"),
        "{}",
        duplicate.message
    );
}

/// The prepared-facts path (`check_with_facts`, used by `no-mistakes check`)
/// and the standalone path (`check`, the N-API/Rust-API entrypoint) must
/// agree — divergence here is exactly the failure mode a partial refactor of
/// the four settings-resolution call sites would produce (silently zero
/// findings on one path).
#[test]
fn check_and_check_with_facts_agree() {
    let (fixture, config_path, config) = load("bound.no-mistakes.yml");
    let root = fixture.path().canonicalize().unwrap();
    let via_check = check(&root, Some(&config_path), &config).unwrap();
    let facts = CheckFactMap::default();
    let via_facts = check_with_facts(&root, Some(&config_path), &config, &facts).unwrap();
    assert_eq!(via_check, via_facts);
}
