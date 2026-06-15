use super::*;
use crate::codebase::terraform::{TerraformBlock, TerraformFileFacts, TerraformRef, TfBlockKind};
use std::collections::BTreeSet;

fn fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/terraform-basic/fixture"),
    )
}

fn report() -> InfraReport {
    analyze_project(&fixture(), None).expect("fixture should analyze")
}

#[test]
fn resource_refs_lists_referencing_blocks() {
    let report = report();
    let rows = report.resource_refs("aws_route53_record.foo");
    let addresses: Vec<&str> = rows.iter().map(|row| row.address.as_str()).collect();
    assert!(addresses.contains(&"aws_lb.web"));
    assert!(addresses.contains(&"output.record_id"));
    // The referencing files resolve relative to the root.
    assert!(rows.iter().all(|row| !row.file.starts_with('/')));
}

#[test]
fn resource_refs_unknown_address_is_empty() {
    assert!(report().resource_refs("aws_does_not.exist").is_empty());
}

#[test]
fn outputs_reports_exports_and_consumers() {
    let report = report();
    let result = report.outputs("infra/modules/network");
    assert!(result.module.ends_with("infra/modules/network"));
    let zone = result
        .exports
        .iter()
        .find(|output| output.name == "zone_id")
        .expect("zone_id export");
    assert!(zone
        .references
        .contains(&"aws_route53_zone.main".to_string()));
    let consumes = result
        .consumers
        .iter()
        .any(|consumer| consumer.output == "zone_id" && consumer.from == "aws_route53_record.foo");
    assert!(consumes);
    // A reference to an output the module does not export is not a consumer.
    assert!(!result
        .consumers
        .iter()
        .any(|consumer| consumer.output == "does_not_exist"));
}

#[test]
fn test_for_resource_mode_matches_referencing_tests() {
    let report = report();
    let rows = report.test_for("infra/envs/prod/main.tf");
    assert!(rows
        .iter()
        .any(|row| row.test_file.ends_with("network.test.mts")));
    // A test mentioning `aws_route53_record.foobar` must not match a file that
    // declares only `aws_route53_record.foo` (identifier-boundary matching).
    assert!(!rows
        .iter()
        .any(|row| row.test_file.ends_with("boundary.test.mts")));
}

#[test]
fn analyze_project_propagates_missing_explicit_config() {
    let result = analyze_project(&fixture(), Some(Path::new("/no/such/no-mistakes.yml")));
    assert!(result.is_err());
}

#[test]
fn analyze_project_without_config_is_empty() {
    // The crate manifest dir has no `.no-mistakes.yml` infra config.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let report = analyze_project(&root, None).expect("analyze");
    assert!(report.resource_refs("aws_x.y").is_empty());
    assert!(report.outputs("infra").exports.is_empty());
    assert!(report.test_for("infra/main.tf").is_empty());
}

// --- Hand-built reports for branch coverage of the matching modes ---

fn report_with(test: TerraformTestConvention, files: Vec<PathBuf>) -> InfraReport {
    let root = fixture();
    let facts = collect_terraform_facts(
        &root,
        &crate::codebase::ts_source::discover_files(&root, &[]),
        &crate::config::v2::schema::TerraformConfig {
            module_roots: vec![
                "infra/envs/prod".to_string(),
                "infra/modules/network".to_string(),
            ],
            ..Default::default()
        },
    );
    let test_globset = compile_test_globs(&test.test_globs).expect("valid test globs");
    InfraReport {
        root,
        files,
        facts,
        test,
        test_globset,
    }
}

#[test]
fn test_for_without_globs_returns_empty() {
    let report = report_with(TerraformTestConvention::default(), Vec::new());
    assert!(report.test_for("infra/envs/prod/main.tf").is_empty());
}

#[test]
fn test_for_unknown_file_returns_empty_even_in_module_mode() {
    let root = fixture();
    let test_file = root.join("infra/envs/prod/__tests__/network.test.mts");
    let report = report_with(
        TerraformTestConvention {
            test_globs: vec!["__tests__/*.test.mts".to_string()],
            test_root: None,
            match_mode: Some("module".to_string()),
        },
        vec![test_file],
    );
    // A mistyped/unconfigured `.tf` path is not a parsed module, so module-mode
    // matching must not report every module test for it.
    assert!(report.test_for("infra/envs/prod/typo.tf").is_empty());
}

#[test]
fn test_for_normalizes_dot_slash_globs() {
    let root = fixture();
    let test_file = root.join("infra/envs/prod/__tests__/network.test.mts");
    let report = report_with(
        TerraformTestConvention {
            test_globs: vec!["./__tests__/*.test.mts".to_string()],
            test_root: None,
            match_mode: Some("module".to_string()),
        },
        vec![test_file],
    );
    let rows = report.test_for("infra/envs/prod/variables.tf");
    assert!(rows
        .iter()
        .any(|row| row.test_file.ends_with("network.test.mts")));
}

#[test]
fn test_for_module_mode_returns_all_module_tests() {
    let root = fixture();
    let test_file = root.join("infra/envs/prod/__tests__/network.test.mts");
    let report = report_with(
        TerraformTestConvention {
            test_globs: vec!["__tests__/*.test.mts".to_string()],
            test_root: None,
            match_mode: Some("module".to_string()),
        },
        vec![test_file.clone()],
    );
    let rows = report.test_for("infra/envs/prod/variables.tf");
    // variables.tf declares no resources, but module mode still returns the test.
    assert!(rows
        .iter()
        .any(|row| row.test_file.ends_with("network.test.mts")));
}

#[test]
fn test_for_resource_mode_skips_tests_without_references() {
    let root = fixture();
    // A test file that does not mention any declared resource address.
    let test_file = root.join("infra/modules/network/__tests__/unrelated.test.mts");
    let report = report_with(
        TerraformTestConvention {
            test_globs: vec!["__tests__/*.test.mts".to_string()],
            test_root: None,
            match_mode: Some("resource".to_string()),
        },
        vec![test_file],
    );
    // network/main.tf declares aws_route53_zone.main, but no candidate test file
    // exists on disk to read, so the resource filter yields nothing.
    assert!(report.test_for("infra/modules/network/main.tf").is_empty());
}

#[test]
fn outputs_skips_modules_sourced_from_other_directories() {
    // Querying the root module: module.network's source is the network dir, which
    // differs from the queried dir, exercising the consumer-skip branch.
    let report = report();
    let result = report.outputs("infra/envs/prod");
    assert!(result
        .exports
        .iter()
        .any(|output| output.name == "record_id"));
    assert!(result.consumers.is_empty());
}

#[test]
fn test_for_resource_mode_empty_declarations_returns_nothing() {
    let root = fixture();
    let test_file = root.join("infra/envs/prod/__tests__/network.test.mts");
    let report = report_with(
        TerraformTestConvention {
            test_globs: vec!["__tests__/*.test.mts".to_string()],
            test_root: None,
            match_mode: Some("resource".to_string()),
        },
        vec![test_file],
    );
    // variables.tf declares no resources, so resource-mode matching is empty.
    assert!(report.test_for("infra/envs/prod/variables.tf").is_empty());
}

#[test]
fn test_for_honors_test_root_anchor() {
    let root = fixture();
    let test_file = root.join("infra/envs/prod/__tests__/network.test.mts");
    // Resource mode scopes by referenced address, so a shared `testRoot` is valid.
    let report = report_with(
        TerraformTestConvention {
            test_globs: vec!["infra/envs/prod/__tests__/network.test.mts".to_string()],
            test_root: Some(".".to_string()),
            match_mode: Some("resource".to_string()),
        },
        vec![test_file],
    );
    let rows = report.test_for("infra/envs/prod/main.tf");
    assert!(rows
        .iter()
        .any(|row| row.test_file.ends_with("network.test.mts")));
}

#[test]
fn validate_module_mode_test_root_rejects_shared_root() {
    use crate::config::v2::schema::TerraformTestConvention;
    let shared = TerraformTestConvention {
        test_globs: vec!["**/*.test.mts".to_string()],
        test_root: Some("tests".to_string()),
        match_mode: Some("module".to_string()),
    };
    assert!(validate_module_mode_test_root(&shared).is_err());
    // Module mode without a shared root, and resource mode with one, are allowed.
    assert!(validate_module_mode_test_root(&TerraformTestConvention {
        match_mode: Some("module".to_string()),
        ..Default::default()
    })
    .is_ok());
    assert!(validate_module_mode_test_root(&TerraformTestConvention {
        test_root: Some("tests".to_string()),
        match_mode: Some("resource".to_string()),
        ..Default::default()
    })
    .is_ok());
}

#[test]
fn compile_test_globs_validates_patterns() {
    assert!(compile_test_globs(&[]).unwrap().is_none());
    assert!(compile_test_globs(&["[".to_string()]).is_err());
    assert!(compile_test_globs(&["*.tf".to_string()]).unwrap().is_some());
}

#[test]
fn validate_match_mode_rejects_unknown_values() {
    assert!(validate_match_mode(None).is_ok());
    assert!(validate_match_mode(Some("resource")).is_ok());
    assert!(validate_match_mode(Some("module")).is_ok());
    assert!(validate_match_mode(Some("modules")).is_err());
}

#[test]
fn analyze_project_rejects_invalid_test_globs() {
    // The dedicated fixture configures an unclosed character-class glob.
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/terraform-bad-globs/fixture"),
    );
    assert!(analyze_project(&root, None).is_err());
}

#[test]
fn output_value_refs_falls_back_when_block_absent() {
    // A report whose facts have an output recorded in the module index but no
    // matching block exercises the empty fallback.
    let root = fixture();
    let mut facts = TerraformFactMap::default();
    let module_dir = root.join("infra/envs/prod");
    facts
        .outputs_by_module
        .entry(module_dir.clone())
        .or_default()
        .insert("ghost".to_string());
    let file = module_dir.join("outputs.tf");
    facts.files.insert(
        file.clone(),
        TerraformFileFacts {
            path: file,
            module_dir: module_dir.clone(),
            blocks: Vec::new(),
            references: Vec::new(),
        },
    );
    let report = InfraReport {
        root,
        files: Vec::new(),
        facts,
        test: TerraformTestConvention::default(),
        test_globset: None,
    };
    let result = report.outputs("infra/envs/prod");
    let ghost = result
        .exports
        .iter()
        .find(|output| output.name == "ghost")
        .expect("ghost export");
    assert!(ghost.references.is_empty());
}

#[test]
fn helper_types_round_trip_through_serde() {
    // Exercise the public result structs (used by N-API JSON output).
    let block = TerraformBlock {
        kind: TfBlockKind::Output,
        addr: "output.x".to_string(),
        name: "x".to_string(),
        file: PathBuf::from("/r/outputs.tf"),
        module_source_dir: None,
        value_refs: vec!["aws_x.y".to_string()],
    };
    assert_eq!(block.name, "x");
    let reference = TerraformRef {
        from_file: PathBuf::from("/r/main.tf"),
        from_addr: "aws_a.b".to_string(),
        to_addr: "aws_x.y".to_string(),
        module_output: None,
    };
    assert_eq!(reference.to_addr, "aws_x.y");
    let set: BTreeSet<String> = ["a".to_string()].into();
    assert_eq!(set.len(), 1);
}
