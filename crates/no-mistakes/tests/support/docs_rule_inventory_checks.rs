use super::rule_docs_helpers::{read, repo_root, rule_ids_from_source};

#[test]
fn ast_grep_ignore_freshness_documents_tracked_inventory_controls() {
    let rule = no_mistakes::codebase::rules::AST_GREP_IGNORE_FRESHNESS;
    assert_eq!(rule, "ast-grep-ignore-freshness");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    let body = read(&repo_root().join(format!("docs/rules/{rule}.md")));
    for phrase in [
        "files",
        "sgconfig.yml",
        "tracked",
        "untracked",
        "dotfile",
        "glob",
        "---",
        "flow",
        "directory",
    ] {
        assert!(body.contains(phrase), "missing {phrase}");
    }
}

#[test]
fn github_actions_action_input_policy_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::GITHUB_ACTIONS_ACTION_INPUT_POLICY;
    assert_eq!(rule, "github-actions-action-input-policy");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    let body = read(&repo_root().join(format!("docs/rules/{rule}.md")));
    for option in ["actions:", "uses:", "require:", "ban:", "include:"] {
        assert!(body.contains(option), "missing {option}");
    }
}

#[test]
fn github_actions_env_policy_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::GITHUB_ACTIONS_ENV_POLICY;
    assert_eq!(rule, "github-actions-env-policy");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    let body = read(&repo_root().join(format!("docs/rules/{rule}.md")));
    for option in ["required:", "forbidden:", "include"] {
        assert!(body.contains(option), "missing {option}");
    }
}

#[test]
fn github_actions_job_permissions_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::GITHUB_ACTIONS_JOB_PERMISSIONS;
    assert_eq!(rule, "github-actions-job-permissions");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    assert!(repo_root().join(format!("docs/rules/{rule}.md")).is_file());
}

#[test]
fn github_actions_required_pr_path_filters_are_documented() {
    let rule = no_mistakes::codebase::rules::GITHUB_ACTIONS_NO_REQUIRED_PR_PATH_FILTERS;
    assert_eq!(rule, "github-actions-no-required-pr-path-filters");
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    assert!(root.join(format!("docs/rules/{rule}.md")).is_file());
    for path in [
        "docs/rules/README.md",
        "docs/cli/check.md",
        "docs/node-api.md",
        "docs/configuration/rules.md",
    ] {
        assert!(read(&root.join(path)).contains(rule), "{path}");
    }
}

#[test]
fn column_check_constraints_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::POSTGRES_COLUMN_CHECK_CONSTRAINTS;
    assert_eq!(rule, "postgres-column-check-constraints");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    let body = read(&repo_root().join(format!("docs/rules/{rule}.md")));
    for option in [
        "schemaCatalogPath",
        "columns:",
        "pattern:",
        "types:",
        "require:",
        "severity: advisory",
        "allow:",
        "reason:",
    ] {
        assert!(body.contains(option), "missing {option}");
    }
}

#[test]
fn postgres_pool_error_listener_rule_has_public_documentation() {
    let page = "postgres-pool-requires-error-listener.md";
    let docs = repo_root().join("docs/eslint-rules");
    assert!(read(&docs.join("README.md")).contains(page));
    let body = read(&docs.join(page));
    for contract in [
        "extraModules",
        "once",
        "addListener",
        "no-mistakes-disable-file",
    ] {
        assert!(body.contains(contract), "missing {contract}");
    }
}

#[test]
fn github_actions_no_insta_snapshot_updates_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::GITHUB_ACTIONS_NO_INSTA_SNAPSHOT_UPDATES;
    assert_eq!(rule, "github-actions-no-insta-snapshot-updates");
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    let body = read(&root.join(format!("docs/rules/{rule}.md")));
    for phrase in ["off by default", "INSTA_UPDATE", "## Suppression"] {
        assert!(body.contains(phrase), "missing {phrase}");
    }
}

#[test]
fn swift_darwin_imports_are_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::SWIFT_NO_UNGUARDED_DARWIN_IMPORT;
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    let page = format!("{rule}.md");
    assert!(root.join("docs/rules").join(&page).is_file());
    for entry in [
        "docs/rules/README.md",
        "docs/cli/check.md",
        "docs/node-api.md",
        "docs/configuration/rules.md",
    ] {
        assert!(read(&root.join(entry)).contains(&page), "{entry}");
    }
}

#[test]
fn terraform_iam_wildcard_rule_is_documented() {
    let rule = no_mistakes::codebase::rules::TERRAFORM_IAM_NO_UNDOCUMENTED_WILDCARDS;
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    assert!(root.join(format!("docs/rules/{rule}.md")).is_file());
    for path in [
        "docs/rules/README.md",
        "docs/configuration/rules.md",
        "docs/cli/check.md",
        "docs/node-api.md",
        "docs/feature-parity.md",
        "plugins/no-mistakes/skills/no-mistakes/references/limits-and-fallbacks.md",
    ] {
        assert!(read(&root.join(path)).contains(rule), "{path}");
    }
}

#[test]
fn native_control_nesting_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::REACT_NO_NESTED_BUTTONS_AND_LINKS;
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    for path in [
        "docs/rules/README.md",
        "docs/rules/react-no-nested-buttons-and-links.md",
        "docs/configuration/rules.md",
        "docs/cli/check.md",
        "docs/node-api.md",
        "docs/feature-parity.md",
        "plugins/no-mistakes/skills/no-mistakes/references/limits-and-fallbacks.md",
    ] {
        assert!(read(&root.join(path)).contains(rule), "{path}");
    }
}

#[test]
fn package_json_export_target_prefix_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::PACKAGE_JSON_EXPORT_TARGET_PREFIX;
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    assert!(root.join(format!("docs/rules/{rule}.md")).is_file());
    for path in [
        "docs/rules/README.md",
        "docs/cli/check.md",
        "docs/configuration/rules.md",
        "docs/node-api.md",
        "docs/feature-parity.md",
        "plugins/no-mistakes/skills/no-mistakes/references/limits-and-fallbacks.md",
    ] {
        assert!(read(&root.join(path)).contains(rule), "{path}");
    }
}

#[test]
fn github_actions_cache_setup_policy_is_in_the_public_inventory() {
    let rule = no_mistakes::codebase::rules::GITHUB_ACTIONS_CACHE_SETUP_POLICY;
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    assert!(root.join(format!("docs/rules/{rule}.md")).is_file());
    for file in [
        "docs/configuration/rules.md",
        "docs/node-api.md",
        "plugins/no-mistakes-rules/skills/github-actions-rules/SKILL.md",
    ] {
        assert!(read(&root.join(file)).contains(rule), "{file}");
    }
}

#[test]
fn postgres_no_duplicate_keyed_lookups_is_documented() {
    let rule = no_mistakes::codebase::rules::POSTGRES_NO_DUPLICATE_KEYED_LOOKUPS;
    assert_eq!(rule, "postgres-no-duplicate-keyed-lookups");
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    let body = read(&root.join(format!("docs/rules/{rule}.md")));
    for phrase in [
        "off by default",
        "sqlInclude",
        "unanalyzable",
        "## Valid example",
        "## Counterexample",
        "## Fix",
        "## Suppression",
    ] {
        assert!(body.contains(phrase), "missing {phrase}");
    }
    for path in [
        "docs/rules/README.md",
        "docs/configuration/rules.md",
        "docs/cli/check.md",
        "docs/node-api.md",
        "docs/feature-parity.md",
    ] {
        assert!(read(&root.join(path)).contains(rule), "{path}");
    }
}

#[test]
fn postgres_lock_reason_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::POSTGRES_LOCK_REASON;
    assert_eq!(rule, "postgres-lock-reason");
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    assert!(root.join(format!("docs/rules/{rule}.md")).is_file());
    assert!(root
        .join(format!(
            "plugins/no-mistakes/skills/no-mistakes/references/{rule}.md"
        ))
        .is_file());
    for entry in [
        "docs/rules/README.md",
        "docs/cli/check.md",
        "docs/node-api.md",
        "docs/configuration/rules.md",
        "plugins/no-mistakes/skills/no-mistakes/SKILL.md",
    ] {
        assert!(read(&root.join(entry)).contains(rule), "{entry}");
    }
}

#[test]
fn queue_rate_limit_error_delay_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::QUEUE_RATE_LIMIT_ERROR_DELAY;
    assert_eq!(rule, "queue-rate-limit-error-delay");
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    let page = format!("{rule}.md");
    let body = read(&root.join("docs/rules").join(&page));
    for phrase in [
        "options.targets",
        "delayArgument",
        "delayProperty",
        "unanalyzable",
        "## Suppression",
    ] {
        assert!(body.contains(phrase), "missing {phrase}");
    }
    for entry in [
        "docs/rules/README.md",
        "docs/cli/check.md",
        "docs/node-api.md",
        "docs/configuration/rules.md",
        "docs/feature-parity.md",
        "plugins/no-mistakes/skills/no-mistakes/references/limits-and-fallbacks.md",
    ] {
        assert!(read(&root.join(entry)).contains(rule), "{entry}");
    }
}
