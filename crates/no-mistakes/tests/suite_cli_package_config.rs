//! Package and config rule JSON snapshots, linked in one harness.

#[path = "common/check_json_snapshot.rs"]
mod check_json_snapshot;

#[path = "cli_pkgcfg_snapshot_config_path_references.rs"]
mod cli_pkgcfg_snapshot_config_path_references;
#[path = "cli_pkgcfg_snapshot_forbidden_calls.rs"]
mod cli_pkgcfg_snapshot_forbidden_calls;
#[path = "cli_pkgcfg_snapshot_forbidden_dependencies.rs"]
mod cli_pkgcfg_snapshot_forbidden_dependencies;
#[path = "cli_pkgcfg_snapshot_forbidden_lint_suppressions.rs"]
mod cli_pkgcfg_snapshot_forbidden_lint_suppressions;
#[path = "cli_pkgcfg_snapshot_forbidden_workspace_closure.rs"]
mod cli_pkgcfg_snapshot_forbidden_workspace_closure;
#[path = "cli_pkgcfg_snapshot_package_json_nested_workspace_coverage.rs"]
mod cli_pkgcfg_snapshot_package_json_nested_workspace_coverage;
#[path = "cli_pkgcfg_snapshot_package_json_registry_only.rs"]
mod cli_pkgcfg_snapshot_package_json_registry_only;
#[path = "cli_pkgcfg_snapshot_package_json_required_fields.rs"]
mod cli_pkgcfg_snapshot_package_json_required_fields;
#[path = "cli_pkgcfg_snapshot_package_json_workspace_coverage.rs"]
mod cli_pkgcfg_snapshot_package_json_workspace_coverage;
#[path = "cli_pkgcfg_snapshot_pnpm_overrides_ban.rs"]
mod cli_pkgcfg_snapshot_pnpm_overrides_ban;
#[path = "cli_pkgcfg_snapshot_pnpm_release_age_policy.rs"]
mod cli_pkgcfg_snapshot_pnpm_release_age_policy;
#[path = "cli_pkgcfg_snapshot_tsconfig_alias_folder_mapping.rs"]
mod cli_pkgcfg_snapshot_tsconfig_alias_folder_mapping;
#[path = "cli_pkgcfg_snapshot_tsconfig_file_coverage.rs"]
mod cli_pkgcfg_snapshot_tsconfig_file_coverage;
#[path = "cli_pkgcfg_snapshot_tsconfig_gate_coverage.rs"]
mod cli_pkgcfg_snapshot_tsconfig_gate_coverage;
#[path = "cli_pkgcfg_snapshot_version_pin_consistency.rs"]
mod cli_pkgcfg_snapshot_version_pin_consistency;
#[path = "cli_pkgcfg_snapshot_workspace_package_cycles.rs"]
mod cli_pkgcfg_snapshot_workspace_package_cycles;
