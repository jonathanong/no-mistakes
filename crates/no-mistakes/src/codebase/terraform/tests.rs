use super::parse::parse_source;
use super::*;
use std::path::Path;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/terraform-basic/fixture"),
    )
}

fn fixture_config() -> TerraformConfig {
    TerraformConfig {
        module_roots: vec![
            "infra/envs/prod".to_string(),
            "infra/modules/network".to_string(),
        ],
        ..Default::default()
    }
}

fn collect_fixture() -> TerraformFactMap {
    let root = fixture();
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    collect_terraform_facts(&root, &all_files, &fixture_config())
}

/// Parse an in-memory HCL snippet as if it lived at `/repo/mod/main.tf`.
fn parse(source: &str) -> TerraformFileFacts {
    parse_source(source, Path::new("/repo/mod/main.tf")).expect("snippet should parse")
}

fn to_addrs(facts: &TerraformFileFacts) -> Vec<String> {
    let mut addrs: Vec<String> = facts.references.iter().map(|r| r.to_addr.clone()).collect();
    addrs.sort();
    addrs.dedup();
    addrs
}

#[test]
fn returns_empty_without_module_roots() {
    let facts = collect_terraform_facts(Path::new("/repo"), &[], &TerraformConfig::default());
    assert!(facts.files.is_empty());
}

#[test]
fn returns_empty_when_no_tf_files_match() {
    let root = fixture();
    // A configured-but-empty module root yields no `.tf` files and no work.
    let config = TerraformConfig {
        module_roots: vec!["infra/envs/staging".to_string()],
        ..Default::default()
    };
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    assert!(collect_terraform_facts(&root, &all_files, &config)
        .files
        .is_empty());
}

#[test]
fn indexes_declarations_for_every_block_kind() {
    let facts = collect_fixture();
    for addr in [
        "module.network",
        "aws_route53_record.foo",
        "aws_lb.web",
        "local.is_internal",
        "data.aws_caller_identity.current",
        "output.record_id",
        "var.region",
        "aws_route53_zone.main",
        "output.zone_id",
        "var.zone_name",
    ] {
        assert!(
            facts.declarations.contains_key(addr),
            "missing declaration for {addr}"
        );
    }
}

#[test]
fn refs_to_reverse_index_lists_referencing_blocks() {
    let facts = collect_fixture();
    let refs = facts
        .refs_to
        .get("aws_route53_record.foo")
        .expect("foo should be referenced");
    let from: Vec<&str> = refs.iter().map(|r| r.from_addr.as_str()).collect();
    // Referenced by aws_lb.web (name) and output.record_id (value).
    assert!(from.contains(&"aws_lb.web"));
    assert!(from.contains(&"output.record_id"));
}

#[test]
fn resolves_local_module_source_and_outputs() {
    let facts = collect_fixture();
    let root = fixture();
    let network_dir = root.join("infra/modules/network");
    assert_eq!(
        facts.module_sources.get("module.network"),
        Some(&network_dir)
    );
    assert!(facts
        .outputs_by_module
        .get(&network_dir)
        .is_some_and(|outputs| outputs.contains("zone_id")));
    assert!(facts
        .outputs_by_module
        .get(&root.join("infra/envs/prod"))
        .is_some_and(|outputs| outputs.contains("record_id")));
}

#[test]
fn records_module_output_consumption() {
    let facts = collect_fixture();
    let consumes_zone_id = facts
        .refs_to
        .get("module.network")
        .expect("module.network should be referenced")
        .iter()
        .any(|r| r.module_output.as_deref() == Some("zone_id"));
    assert!(consumes_zone_id);
}

#[test]
fn groups_files_by_module_directory() {
    let facts = collect_fixture();
    let prod = fixture().join("infra/envs/prod");
    let files = facts.files_by_module.get(&prod).expect("prod module files");
    assert!(files.iter().any(|p| p.ends_with("main.tf")));
    assert!(files.iter().any(|p| p.ends_with("outputs.tf")));
}

#[test]
fn discovery_excludes_nested_non_module_directories() {
    let facts = collect_fixture();
    // `infra/modules/network/examples/example.tf` is in a nested directory that is
    // not a configured module root, so it must not be indexed.
    assert!(!facts.declarations.contains_key("aws_example.nested"));
}

#[test]
fn skips_and_warns_on_malformed_files() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/terraform-malformed/fixture"),
    );
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let config = TerraformConfig {
        module_roots: vec!["infra".to_string()],
        ..Default::default()
    };
    // The malformed file is skipped rather than panicking or being indexed.
    let facts = collect_terraform_facts(&root, &all_files, &config);
    assert!(facts.files.is_empty());
}

include!("tests/parse_cases.rs");
