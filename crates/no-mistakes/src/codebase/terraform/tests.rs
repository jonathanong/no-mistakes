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
fn classifies_every_reference_kind() {
    let facts = parse(
        r#"
        resource "aws_instance" "web" {
          ami        = data.aws_ami.ubuntu.id
          subnet_id  = aws_subnet.main.id
          region     = var.region
          name       = local.name
          zone       = module.net.zone_id
          tag        = "${aws_eip.ip.public_ip}"
          index      = count.index
        }
        "#,
    );
    let addrs = to_addrs(&facts);
    assert!(addrs.contains(&"data.aws_ami.ubuntu".to_string()));
    assert!(addrs.contains(&"aws_subnet.main".to_string()));
    assert!(addrs.contains(&"var.region".to_string()));
    assert!(addrs.contains(&"local.name".to_string()));
    assert!(addrs.contains(&"module.net".to_string()));
    assert!(addrs.contains(&"aws_eip.ip".to_string()));
    // `count.index` is a meta-value, not a reference.
    assert!(!addrs.iter().any(|a| a.starts_with("count")));
}

#[test]
fn walks_collection_and_operation_expressions() {
    let facts = parse(
        r#"
        resource "aws_lb" "web" {
          subnets    = [aws_subnet.a.id, aws_subnet.b.id]
          tags       = { owner = var.owner }
          count      = var.enabled ? 1 : 0
          name       = upper(local.prefix)
          combined   = var.a + var.b
          all        = [for s in var.subnets : s.id]
        }
        "#,
    );
    let addrs = to_addrs(&facts);
    for expected in [
        "aws_subnet.a",
        "aws_subnet.b",
        "var.owner",
        "var.enabled",
        "local.prefix",
        "var.a",
        "var.b",
        "var.subnets",
    ] {
        assert!(addrs.contains(&expected.to_string()), "missing {expected}");
    }
}

#[test]
fn walks_template_directives_unary_parenthesis_and_index() {
    let facts = parse(
        r#"
        resource "aws_instance" "web" {
          banner   = "%{ if var.enabled }${aws_eip.ip.public_ip}%{ else }${aws_eip.backup.public_ip}%{ endif }"
          loop_tag = "%{ for s in var.subnets }${aws_subnet.main.id}%{ endfor }"
          flag     = !var.disabled
          neg      = -local.offset
          wrapped  = (var.wrapped)
          indexed  = var.list[local.idx]
          mapped   = { (var.key_expr) = var.value_expr }
          filtered = [for s in var.items : s.id if var.keep]
          obj_for  = { for k in var.keys : k => var.lookup }
        }
        "#,
    );
    let addrs = to_addrs(&facts);
    for expected in [
        "aws_eip.ip",
        "aws_eip.backup",
        "aws_subnet.main",
        "var.enabled",
        "var.subnets",
        "var.disabled",
        "local.offset",
        "var.wrapped",
        "var.list",
        "local.idx",
        "var.key_expr",
        "var.value_expr",
        "var.items",
        "var.keep",
        "var.keys",
        "var.lookup",
    ] {
        assert!(addrs.contains(&expected.to_string()), "missing {expected}");
    }
}

#[test]
fn module_source_skips_non_string_and_missing_sources() {
    // `source` referencing a variable is not a static local path.
    let dynamic = parse(
        r#"
        module "vpc" {
          source = var.module_source
        }
        "#,
    );
    assert!(dynamic.blocks[0].module_source_dir.is_none());

    // A module block with no `source` attribute.
    let missing = parse(
        r#"
        module "vpc" {
          region = var.region
        }
        "#,
    );
    assert!(missing.blocks[0].module_source_dir.is_none());
}

#[test]
fn output_without_value_has_no_value_refs() {
    let facts = parse(
        r#"
        output "empty" {
          description = "no value"
        }
        "#,
    );
    let block = facts
        .blocks
        .iter()
        .find(|b| b.addr == "output.empty")
        .unwrap();
    assert!(block.value_refs.is_empty());
}

#[test]
fn effective_extensions_uses_configured_values() {
    let config = TerraformConfig {
        module_roots: vec!["infra".to_string()],
        extensions: vec!["tofu".to_string()],
        ..Default::default()
    };
    assert_eq!(config.effective_extensions(), vec!["tofu".to_string()]);
}

#[test]
fn module_source_skips_remote_registry_sources() {
    let facts = parse(
        r#"
        module "vpc" {
          source = "terraform-aws-modules/vpc/aws"
        }
        "#,
    );
    let block = facts
        .blocks
        .iter()
        .find(|b| b.addr == "module.vpc")
        .expect("module block");
    assert!(block.module_source_dir.is_none());
}

#[test]
fn ignores_unknown_and_malformed_blocks() {
    // `provider` and `terraform` blocks declare nothing; missing labels are skipped.
    let facts = parse(
        r#"
        terraform {
          required_version = ">= 1.5"
        }
        provider "aws" {
          region = var.region
        }
        resource "aws_s3_bucket" {
          bucket = "x"
        }
        "#,
    );
    assert!(facts.blocks.is_empty());
    // The provider's `var.region` reference is not attributed (no declaring block).
    assert!(facts.references.is_empty());
}

#[test]
fn parse_source_returns_none_for_invalid_hcl() {
    assert!(parse_source("resource \"aws_s3_bucket\" {", Path::new("/repo/x.tf")).is_none());
}

#[test]
fn has_extension_excludes_tf_json() {
    let extensions = vec!["tf".to_string()];
    assert!(has_extension(Path::new("/repo/main.tf"), &extensions));
    assert!(!has_extension(Path::new("/repo/main.tf.json"), &extensions));
    assert!(!has_extension(Path::new("/repo/main.hcl"), &extensions));
}
