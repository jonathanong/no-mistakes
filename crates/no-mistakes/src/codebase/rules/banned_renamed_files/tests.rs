use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn config_with_rule(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn compiled(opts: &Options) -> Vec<CompiledBasename<'_>> {
    compile_basenames(opts).unwrap()
}

fn fixture_root(subpath: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/banned-renamed-files/fixture")
            .join(subpath),
    )
}

#[test]
fn pass_fixture_has_no_findings() {
    let root = fixture_root("pass");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "expected no findings, got: {findings:?}"
    );
}

#[test]
fn fail_fixture_has_findings() {
    let root = fixture_root("fail");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(!findings.is_empty(), "expected findings for banned file");
    assert!(findings[0].message.contains("rename middleware"));
}

#[test]
fn root_scope_fixture_has_findings() {
    let root = fixture_root("root-scope-fail");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "web/middleware.ts");
    assert!(findings[0].message.contains("rename middleware"));
}

#[test]
fn prefix_fixture_has_no_findings() {
    let root = fixture_root("prefix-pass");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "web scope should not match web2: {findings:?}"
    );
}

#[test]
fn compound_fixture_flags_multipart_variants() {
    let root = fixture_root("compound-fail");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    let files: Vec<&str> = findings.iter().map(|f| f.file.as_str()).collect();
    assert_eq!(
        files,
        vec![
            "web/webpack.config.dev.ts",
            "web/webpack.config.js",
            "web/webpack.config.prod.js",
        ],
        "compound mode should flag single- and multi-part suffixes but not webpack.configuration.js"
    );
}

#[test]
fn regex_fixture_flags_pattern_matches() {
    let root = fixture_root("regex-fail");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    let files: Vec<&str> = findings.iter().map(|f| f.file.as_str()).collect();
    assert_eq!(
        files,
        vec!["web/jest.config.cjs", "web/jest.config.js"],
        "regex `pattern` should match every jest.config.* variant"
    );
}

fn opts_with_middleware() -> Options {
    Options {
        scope: Some("web".to_string()),
        banned_basenames: vec![BannedBasename {
            name: "middleware".to_string(),
            message: "rename middleware.{ts,mts,js} to proxy.ts".to_string(),
            ..Default::default()
        }],
        extensions: vec![".ts".to_string(), ".mts".to_string(), ".js".to_string()],
    }
}

#[test]
fn banned_basename_in_scope_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("web")).unwrap();
    let path = root.join("web/middleware.ts");
    std::fs::write(&path, "export {};\n").unwrap();
    let opts = opts_with_middleware();
    let findings = check_file(&path, root, &opts, &compiled(&opts));
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("rename"));
}

#[test]
fn non_banned_name_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("web")).unwrap();
    let path = root.join("web/proxy.ts");
    std::fs::write(&path, "export {};\n").unwrap();
    let opts = opts_with_middleware();
    let findings = check_file(&path, root, &opts, &compiled(&opts));
    assert!(findings.is_empty());
}

#[test]
fn out_of_scope_path_not_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend")).unwrap();
    let path = root.join("backend/middleware.ts");
    std::fs::write(&path, "export {};\n").unwrap();
    let opts = opts_with_middleware();
    let findings = check_file(&path, root, &opts, &compiled(&opts));
    assert!(
        findings.is_empty(),
        "out-of-scope path should not be flagged"
    );
}

#[test]
fn non_matching_extension_not_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("web")).unwrap();
    let path = root.join("web/middleware.py");
    std::fs::write(&path, "# python\n").unwrap();
    let opts = opts_with_middleware();
    let findings = check_file(&path, root, &opts, &compiled(&opts));
    assert!(findings.is_empty(), ".py should not be flagged");
}

fn opts_compound() -> Options {
    Options {
        scope: None,
        banned_basenames: vec![BannedBasename {
            name: "webpack.config".to_string(),
            message: "remove webpack config".to_string(),
            match_compound_extensions: true,
            ..Default::default()
        }],
        extensions: vec![".ts".to_string(), ".js".to_string()],
    }
}

#[test]
fn compound_flags_single_and_multipart() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let opts = opts_compound();
    let c = compiled(&opts);
    for name in [
        "webpack.config.js",
        "webpack.config.prod.js",
        "webpack.config.dev.ts",
    ] {
        let path = root.join(name);
        std::fs::write(&path, "export {};\n").unwrap();
        assert_eq!(
            check_file(&path, root, &opts, &c).len(),
            1,
            "{name} should be flagged in compound mode"
        );
    }
}

#[test]
fn compound_does_not_match_unrelated_prefix() {
    // `webpack.configuration.js` shares a prefix but is not `webpack.config` + extensions.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let opts = opts_compound();
    let path = root.join("webpack.configuration.js");
    std::fs::write(&path, "export {};\n").unwrap();
    assert!(check_file(&path, root, &opts, &compiled(&opts)).is_empty());
}

#[test]
fn compound_still_respects_extensions() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let opts = opts_compound();
    let path = root.join("webpack.config.prod.py");
    std::fs::write(&path, "# python\n").unwrap();
    assert!(
        check_file(&path, root, &opts, &compiled(&opts)).is_empty(),
        ".py is not in extensions even in compound mode"
    );
}

fn opts_regex(pattern: &str) -> Options {
    Options {
        scope: None,
        banned_basenames: vec![BannedBasename {
            name: "jest.config".to_string(),
            message: "remove jest config".to_string(),
            pattern: Some(pattern.to_string()),
            ..Default::default()
        }],
        // extensions intentionally narrow — regex entries ignore the extension filter.
        extensions: vec![".ts".to_string()],
    }
}

#[test]
fn regex_matches_basename_ignoring_extensions_filter() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let opts = opts_regex(r"^jest\.config\..+");
    let c = compiled(&opts);
    let path = root.join("jest.config.cjs");
    std::fs::write(&path, "module.exports = {};\n").unwrap();
    assert_eq!(
        check_file(&path, root, &opts, &c).len(),
        1,
        "regex matches even though .cjs is not in extensions"
    );
}

#[test]
fn regex_non_match_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let opts = opts_regex(r"^jest\.config\..+");
    let path = root.join("vitest.config.ts");
    std::fs::write(&path, "export default {};\n").unwrap();
    assert!(check_file(&path, root, &opts, &compiled(&opts)).is_empty());
}

#[test]
fn invalid_regex_pattern_errors() {
    let opts = opts_regex(r"(unclosed");
    let err = match compile_basenames(&opts) {
        Ok(_) => panic!("expected invalid regex to error"),
        Err(err) => err,
    };
    let msg = format!("{err:#}");
    assert!(msg.contains("invalid `pattern` regex"), "got: {msg}");
    assert!(
        msg.contains("jest.config"),
        "error names the basename: {msg}"
    );
}

#[test]
fn no_scope_matches_all_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("anywhere")).unwrap();
    let path = root.join("anywhere/middleware.ts");
    std::fs::write(&path, "export {};\n").unwrap();
    let opts = Options {
        scope: None,
        banned_basenames: vec![BannedBasename {
            name: "middleware".to_string(),
            message: "rename it".to_string(),
            ..Default::default()
        }],
        extensions: vec![".ts".to_string()],
    };
    let findings = check_file(&path, root, &opts, &compiled(&opts));
    assert_eq!(findings.len(), 1);
}

#[test]
fn check_with_files_works() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("web")).unwrap();
    let path = root.join("web/middleware.mts");
    std::fs::write(&path, "export {};\n").unwrap();
    let yaml = "scope: web\nbannedBasenames:\n  - name: middleware\n    message: rename it\nextensions: [\".mts\"]";
    let config = config_with_rule(yaml);
    let findings = check_with_files(root, &config, &[path]).unwrap();
    assert_eq!(findings.len(), 1);
}

#[test]
fn file_with_no_name_component_returns_empty() {
    // A path with no file_name (e.g. the root itself) should return no findings.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let opts = Options {
        scope: None,
        banned_basenames: vec![BannedBasename {
            name: "middleware".to_string(),
            message: "rename it".to_string(),
            ..Default::default()
        }],
        extensions: vec![".ts".to_string()],
    };
    // Pass the root dir itself — has no file_name in the sense of "middleware"
    let findings = check_file(root, root, &opts, &compiled(&opts));
    assert!(
        findings.is_empty(),
        "root path (no matching stem) should produce no findings"
    );
}

#[test]
fn file_with_no_dot_not_flagged_for_dot_extension() {
    // A file with no dot in its name: stem == filename, ext == ""
    // The dot_ext would be "." which won't match any extension like ".ts"
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let path = root.join("middleware"); // no extension — split_stem_ext returns ("middleware", "")
    std::fs::write(&path, "#!/bin/bash\n").unwrap();
    let opts = Options {
        scope: None,
        banned_basenames: vec![BannedBasename {
            name: "middleware".to_string(),
            message: "rename it".to_string(),
            ..Default::default()
        }],
        extensions: vec![".ts".to_string()],
    };
    let findings = check_file(&path, root, &opts, &compiled(&opts));
    assert!(
        findings.is_empty(),
        "file with no extension should not match banned ext '.ts'"
    );
}

#[test]
fn path_with_no_file_name_returns_empty() {
    // On Unix, Path::new("/").file_name() returns None, exercising line 81.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let opts = Options {
        scope: None,
        banned_basenames: vec![BannedBasename {
            name: "middleware".to_string(),
            message: "rename it".to_string(),
            ..Default::default()
        }],
        extensions: vec![".ts".to_string()],
    };
    // Use the filesystem root — file_name() is None for paths ending in "/"
    let findings = check_file(std::path::Path::new("/"), root, &opts, &compiled(&opts));
    assert!(
        findings.is_empty(),
        "path with no file_name should return no findings"
    );
}
