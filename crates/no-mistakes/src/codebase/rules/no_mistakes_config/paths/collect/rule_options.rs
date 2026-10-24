use super::{path_kind, push, Field, Kind, Ref};
use crate::config::v2::NoMistakesConfig;
use serde_yaml::Value;

pub(super) fn collect(config: &NoMistakesConfig, refs: &mut Vec<Ref>, origins: bool) {
    for (index, rule) in config.rules.iter().enumerate() {
        let field = Field::new("rules", origins).index(index).key("options");
        collect_rule_paths(&rule.rule, &rule.options, &field, refs);
    }
}

// Rule options are deliberately not traversed by key name. Several rules use
// names such as `allowlist` for package identities rather than filesystem
// paths, so only each rule's declared path-bearing schema fields belong here.
fn collect_rule_paths(rule: &str, value: &Value, field: &Field, refs: &mut Vec<Ref>) {
    match rule {
        "agents-md-max-size"
        | "required-local-docs"
        | "require-test-per-subdir"
        | "rust-max-lines-per-file"
        | "rust-no-inline-allows" => {
            paths_at(value, "roots", field, Kind::Directory, refs);
        }
        "csharp-max-lines-per-file" => {
            paths_at(value, "roots", field, Kind::Directory, refs);
            paths_at(value, "testRoots", field, Kind::Glob, refs);
        }
        "doc-consistency" => {
            paths_at(value, "requiredFiles", field, Kind::File, refs);
            object_paths_at(value, "requiredSubstrings", "file", field, Kind::File, refs);
            banned_pattern_globs(value, field, refs);
        }
        "file-extension-policy" => {
            paths_at(value, "allowlist", field, Kind::File, refs);
            object_paths_at(value, "scopes", "path", field, Kind::Directory, refs);
        }
        "forbidden-dependencies" => {
            paths_at(value, "roots", field, Kind::Directory, refs);
            paths_at(value, "forbiddenFiles", field, Kind::File, refs);
        }
        "forbidden-workspace-closure" => paths_at(value, "lockfile", field, Kind::File, refs),
        "markdown-reachability" | "markdown-structure-budget" => {
            paths_at(value, "baselineFile", field, Kind::File, refs);
        }
        "nextjs-redirect-destinations" => {
            paths_at(value, "configPath", field, Kind::File, refs);
            paths_at(value, "appRoot", field, Kind::Directory, refs);
        }
        "package-json-registry-only" => {
            paths_at(value, "lockfile", field, Kind::File, refs);
            paths_at(value, "scopes", field, Kind::Directory, refs);
        }
        "package-json-nested-workspace-coverage" => {
            paths_at(value, "roots", field, Kind::Directory, refs);
        }
        "package-json-workspace-coverage" => {
            paths_at(value, "packageRoots", field, Kind::Directory, refs);
            paths_at(value, "allowlist", field, Kind::File, refs);
        }
        "pnpm-release-age-policy" => {
            paths_at(value, "workspaceYaml", field, Kind::File, refs);
            paths_at(value, "dependabotPath", field, Kind::File, refs);
            paths_at(value, "lockfilePath", field, Kind::File, refs);
        }
        "shellcheck-runner" => {
            paths_at(value, "shellFiles", field, Kind::File, refs);
            paths_at(value, "shebangDirs", field, Kind::Directory, refs);
            paths_at(value, "skillsLockfile", field, Kind::File, refs);
        }
        "strict-package-layout" => {
            object_paths_at(value, "packages", "root", field, Kind::Directory, refs);
        }
        "tsconfig-alias-folder-mapping" => {
            paths_at(value, "tsconfig", field, Kind::File, refs);
            paths_at(value, "baseDir", field, Kind::Directory, refs);
        }
        "tsconfig-file-coverage" => {
            object_paths_at(value, "allow", "path", field, Kind::File, refs);
            object_paths_at(value, "auxiliaryConfigs", "path", field, Kind::File, refs);
        }
        _ => {}
    }
}

fn banned_pattern_globs(value: &Value, field: &Field, refs: &mut Vec<Ref>) {
    let Some(patterns) = value.get("bannedPatterns").and_then(Value::as_sequence) else {
        return;
    };
    for (index, item) in patterns.iter().enumerate() {
        let Some(files) = item.get("files") else {
            continue;
        };
        for (file_index, (authored, path)) in string_values(files).into_iter().enumerate() {
            push_path(
                refs,
                field
                    .key("bannedPatterns")
                    .index(index)
                    .key("files")
                    .index_at(file_index, authored),
                Kind::Glob,
                path,
            );
        }
    }
}

fn paths_at(value: &Value, key: &str, field: &Field, kind: Kind, refs: &mut Vec<Ref>) {
    let Some(value) = value.get(key) else {
        return;
    };
    for (index, (authored, path)) in string_values(value).into_iter().enumerate() {
        push_path(refs, field.key(key).index_at(index, authored), kind, path);
    }
}

fn object_paths_at(
    value: &Value,
    key: &str,
    nested_key: &str,
    field: &Field,
    kind: Kind,
    refs: &mut Vec<Ref>,
) {
    let Some(values) = value.get(key).and_then(Value::as_sequence) else {
        return;
    };
    for (index, item) in values.iter().enumerate() {
        let Some(path) = item.get(nested_key).and_then(Value::as_str) else {
            continue;
        };
        push_path(
            refs,
            field.key(key).index(index).key(nested_key),
            kind,
            path,
        );
    }
}

fn push_path(refs: &mut Vec<Ref>, field: Field, kind: Kind, value: &str) {
    let value = value.trim();
    if value.is_empty() || value.starts_with('!') {
        return;
    }
    let kind = if path_kind(value) == Kind::Glob {
        Kind::Glob
    } else {
        kind
    };
    push(refs, field, kind, value);
}

fn string_values(value: &Value) -> Vec<(usize, &str)> {
    match value {
        Value::String(value) => vec![(0, value)],
        Value::Sequence(values) => values
            .iter()
            .enumerate()
            .filter_map(|(index, value)| value.as_str().map(|value| (index, value)))
            .collect(),
        _ => Vec::new(),
    }
}
