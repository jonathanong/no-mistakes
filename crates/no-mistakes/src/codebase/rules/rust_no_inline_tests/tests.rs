use super::*;
use crate::codebase::rules::{path_filter, rust_max_lines_per_file, sort_findings, target_roots};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

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

fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> anyhow::Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = rule.rule_options();
        let roots = normalize_roots(&opts, root, &target_roots(root, config, rule));
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| {
                roots.iter().any(|rule_root| path.starts_with(rule_root))
                    && !is_excluded(root, path, &opts.excludes)
                    && !rust_max_lines_per_file::is_test_file(root, path)
            })
            .cloned()
            .collect();
        let files = path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan(root, &opts, &files)?);
    }
    sort_findings(&mut findings);
    Ok(findings)
}

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/rust-no-inline-tests/fixture")
        .join(path)
}

fn check_fixture(path: &str) -> Vec<RuleFinding> {
    let path = fixture(path);
    check_file(&path, path.parent().unwrap())
}

fn check_source(source: &str) -> Vec<RuleFinding> {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("lib.rs");
    std::fs::write(&path, source).unwrap();
    let root = tmp.path();
    check_file(&path, root)
}

#[test]
fn no_match_on_clean_source() {
    let src = "fn foo() {}\n";
    assert!(check_source(src).is_empty());
}

#[test]
fn no_match_on_out_of_line_tests() {
    let src = "#[cfg(test)]\nmod tests;\n";
    assert!(
        check_source(src).is_empty(),
        "out-of-line mod tests; must not be flagged"
    );
}

#[test]
fn matches_simple_inline() {
    let src = "#[cfg(test)]\nmod tests {\n    fn it_works() {}\n}\n";
    let findings = check_source(src);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 1);
}

#[test]
fn matches_inline_on_one_line() {
    let src = "#[cfg(test)] mod tests { fn it_works() {} }\n";
    let findings = check_source(src);
    assert_eq!(findings.len(), 1);
}

#[test]
fn matches_pub_mod() {
    let src = "#[cfg(test)]\npub mod tests {\n    fn it_works() {}\n}\n";
    let findings = check_source(src);
    assert_eq!(findings.len(), 1);
}

#[test]
fn matches_cfg_test_function() {
    let src = "#[cfg(test)]\npub fn helper() {}\n";
    let findings = check_source(src);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 1);
    assert!(
        findings[0].message.contains("inline #[cfg(test)] item"),
        "{}",
        findings[0].message
    );
}

#[test]
fn matches_cfg_test_use() {
    let src = "#[cfg(test)]\nuse std::collections::HashMap;\n";
    let findings = check_source(src);
    assert_eq!(findings.len(), 1);
}

#[test]
fn matches_cfg_test_impl_item() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/rust-no-inline-tests/fixture/fail-associated");
    let findings = check_file(&root.join("lib.rs"), &root);
    assert_eq!(findings.len(), 3);
    assert_eq!(findings[0].line, 4);
}

#[test]
fn matches_with_whitespace_in_cfg() {
    let src = "# [ cfg ( test ) ]\nmod tests {}\n";
    let findings = check_source(src);
    assert_eq!(findings.len(), 1);
}

#[test]
fn matches_with_extra_attributes() {
    let findings = check_fixture("unit/extra_attrs.rs");
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_cfg_path_attribute_without_tokens() {
    assert!(check_fixture("unit/path_cfg.rs").is_empty());
}

#[test]
fn respects_disable_file_comment() {
    let src = format!("// no-mistakes-disable-file {RULE_ID}\n#[cfg(test)]\nmod tests {{\n}}\n");
    let findings = check_source(&src);
    assert!(findings.is_empty());
}

#[test]
fn check_returns_empty_for_no_rs_files() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_with_rule("{}");
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_reports_correct_file_path() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("mymod.rs");
    std::fs::write(&path, "#[cfg(test)]\nmod tests {\n}\n").unwrap();
    let config = config_with_rule("{}");
    let findings = check(tmp.path(), &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "mymod.rs");
}

#[test]
fn check_respects_excludes() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("generated.rs");
    std::fs::write(&path, "#[cfg(test)]\nmod tests {\n}\n").unwrap();
    let config = config_with_rule("{excludes: [\"generated\"]}");
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_with_files_respects_roots() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let sub = root.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let inline_src = "#[cfg(test)]\nmod tests {\n}\n";
    let outside = root.join("a.rs");
    let inside = sub.join("b.rs");
    std::fs::write(&outside, inline_src).unwrap();
    std::fs::write(&inside, inline_src).unwrap();
    let sub_str = sub.to_str().unwrap();
    let config = config_with_rule(&format!("{{roots: [\"{sub_str}\"]}}"));
    let all_files = vec![outside, inside];
    let findings = check_with_files(root, &config, &all_files).unwrap();
    assert_eq!(
        findings.len(),
        1,
        "only the file within roots should be flagged"
    );
    assert!(findings[0].file.contains("sub"));
}

#[test]
fn check_with_files_normalizes_relative_roots() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let sub = root.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let inline_src = "#[cfg(test)]\nmod tests {\n}\n";
    let outside = root.join("a.rs");
    let inside = sub.join("b.rs");
    std::fs::write(&outside, inline_src).unwrap();
    std::fs::write(&inside, inline_src).unwrap();
    let config = config_with_rule("{roots: [\"sub\"]}");
    let all_files = vec![outside, inside];
    let findings = check_with_files(root, &config, &all_files).unwrap();
    assert_eq!(findings.len(), 1, "relative root resolves relative to root");
    assert!(findings[0].file.contains("sub"));
}

#[test]
fn check_sorts_by_file_then_line() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("a.rs"),
        "#[cfg(test)]\nmod a_tests {}\n#[cfg(test)]\nmod b_tests {}\n",
    )
    .unwrap();
    std::fs::write(tmp.path().join("b.rs"), "#[cfg(test)]\nmod tests {}\n").unwrap();
    let config = config_with_rule("{}");
    let findings = check(tmp.path(), &config).unwrap();
    assert_eq!(findings.len(), 3);
    for i in 1..findings.len() {
        let a = (&findings[i - 1].file, findings[i - 1].line);
        let b = (&findings[i].file, findings[i].line);
        assert!(a <= b);
    }
}
