//! Cli test plans integration tests, linked in one harness.

// Share stateless fixture helpers across the modules in this harness.
#[path = "common/mod.rs"]
mod common;
// This suite uses materialize_saved; materialize is used by the other suites.
#[allow(dead_code)]
#[path = "common/gitignore_fixture.rs"]
mod gitignore_fixture;
#[path = "common/saved_fixture.rs"]
mod saved_fixture;

#[path = "cli_test_plan_config.rs"]
mod cli_test_plan_config;
#[path = "cli_test_plan_config2.rs"]
mod cli_test_plan_config2;
#[path = "cli_test_plan_direct_import.rs"]
mod cli_test_plan_direct_import;
#[path = "cli_test_plan_dotnet.rs"]
mod cli_test_plan_dotnet;
#[path = "cli_test_plan_fallback.rs"]
mod cli_test_plan_fallback;
#[path = "cli_test_plan_lang.rs"]
mod cli_test_plan_lang;
#[path = "cli_test_plan_project_discovery.rs"]
mod cli_test_plan_project_discovery;
#[path = "cli_test_plan_resources.rs"]
mod cli_test_plan_resources;
#[path = "cli_test_plan_sample_when_limited.rs"]
mod cli_test_plan_sample_when_limited;
#[path = "cli_test_plan_selector_grouping.rs"]
mod cli_test_plan_selector_grouping;
#[path = "cli_test_plan_swift_native_fallback.rs"]
mod cli_test_plan_swift_native_fallback;
#[path = "cli_test_plan_swift_native_source.rs"]
mod cli_test_plan_swift_native_source;
#[path = "cli_test_plan_targeted_triggers.rs"]
mod cli_test_plan_targeted_triggers;
#[path = "cli_tests_impact.rs"]
mod cli_tests_impact;
#[path = "cli_tests_impact_diff.rs"]
mod cli_tests_impact_diff;
#[path = "cli_tests_impact_features.rs"]
mod cli_tests_impact_features;
#[path = "cli_tests_impact_native.rs"]
mod cli_tests_impact_native;
#[path = "cli_tests_impact_rendering.rs"]
mod cli_tests_impact_rendering;
#[path = "cli_tests_impact_symbol.rs"]
mod cli_tests_impact_symbol;
#[path = "cli_tests_impact_symbol_extra.rs"]
mod cli_tests_impact_symbol_extra;
#[path = "cli_tests_impact_symbol_review.rs"]
mod cli_tests_impact_symbol_review;
#[path = "cli_tests_impact_symbol_shepherd.rs"]
mod cli_tests_impact_symbol_shepherd;
#[path = "cli_tests_impact_symbol_shepherd2.rs"]
mod cli_tests_impact_symbol_shepherd2;
#[path = "cli_tests_impact_symbol_shepherd3.rs"]
mod cli_tests_impact_symbol_shepherd3;
#[path = "cli_tests_plan_from_git_diff.rs"]
mod cli_tests_plan_from_git_diff;
#[path = "cli_tests_plan_git_diff_hunks.rs"]
mod cli_tests_plan_git_diff_hunks;
#[path = "cli_tests_plan_lockfile.rs"]
mod cli_tests_plan_lockfile;
#[path = "cli_tests_plan_lockfile2.rs"]
mod cli_tests_plan_lockfile2;
#[path = "cli_tests_plan_lockfile3.rs"]
mod cli_tests_plan_lockfile3;
#[path = "cli_tests_targets_json_workspace.rs"]
mod cli_tests_targets_json_workspace;
#[path = "cli_vitest_config_extends.rs"]
mod cli_vitest_config_extends;
#[path = "cli_vitest_project_names.rs"]
mod cli_vitest_project_names;
#[path = "cli_vitest_workspace_precedence.rs"]
mod cli_vitest_workspace_precedence;
