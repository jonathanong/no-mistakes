#[test]
fn binds_dynamic_block_iterator_variables() {
    let facts = parse(
        r#"
        resource "aws_security_group" "web" {
          dynamic "ingress" {
            for_each = var.rules
            content {
              from_port = ingress.value.port
              cidr      = aws_subnet.main.cidr
            }
          }
        }
        "#,
    );
    let addrs = to_addrs(&facts);
    // `for_each` and real references are recorded.
    assert!(addrs.contains(&"var.rules".to_string()));
    assert!(addrs.contains(&"aws_subnet.main".to_string()));
    // The dynamic iterator `ingress` is a local, not an `ingress.value` resource.
    assert!(!addrs.iter().any(|a| a.starts_with("ingress.")));
}

#[test]
fn dynamic_for_each_is_evaluated_in_outer_scope() {
    let facts = parse(
        r#"
        resource "aws_subnet" "web" {
          dynamic "subnet" {
            for_each = subnet.ids
            content {
              cidr = subnet.value.cidr
            }
          }
        }
        "#,
    );
    let addrs = to_addrs(&facts);
    // `for_each` is outside the iterator scope, so `subnet.ids` is a real ref.
    assert!(addrs.contains(&"subnet.ids".to_string()));
    // Inside `content`, `subnet` is the bound iterator, not a resource.
    assert!(!addrs.iter().any(|a| a == "subnet.value"));
}

#[test]
fn binds_explicit_dynamic_iterator_and_template_for_keys() {
    let facts = parse(
        r#"
        resource "aws_security_group" "web" {
          dynamic "ingress" {
            for_each = var.rules
            iterator = rule
            content {
              port = rule.value.port
            }
          }
          banner = "%{ for k, v in var.pairs }${k}=${v} %{ endfor }"
        }
        "#,
    );
    let addrs = to_addrs(&facts);
    assert!(addrs.contains(&"var.rules".to_string()));
    assert!(addrs.contains(&"var.pairs".to_string()));
    // The explicit `iterator = rule` and template `for` keys are locals.
    assert!(!addrs.iter().any(|a| a.starts_with("rule.")));
    assert!(!addrs.iter().any(|a| a == "k" || a == "v"));
}

#[test]
fn ignores_for_expression_iterator_variables() {
    let facts = parse(
        r#"
        resource "aws_lb" "web" {
          ids = [for subnet in aws_subnet.main : subnet.id]
          kv  = { for k, v in var.entries : k => v.id }
        }
        "#,
    );
    let addrs = to_addrs(&facts);
    // The collection expressions are real references.
    assert!(addrs.contains(&"aws_subnet.main".to_string()));
    assert!(addrs.contains(&"var.entries".to_string()));
    // The iterator variables are locals, not resource references.
    assert!(!addrs.iter().any(|a| a.starts_with("subnet.")));
    assert!(!addrs.iter().any(|a| a.starts_with("v.")));
}

#[test]
fn resolves_splatted_module_outputs() {
    let facts = parse(
        r#"
        output "ids" {
          value = module.network[*].zone_id
        }
        "#,
    );
    let block = facts
        .blocks
        .iter()
        .find(|b| b.addr == "output.ids")
        .unwrap();
    assert!(block
        .value_refs
        .contains(&"module.network.zone_id".to_string()));
}

#[test]
fn resolves_bracketed_module_outputs() {
    let facts = parse(
        r#"
        output "ids" {
          value = module.network["zone_id"]
        }
        "#,
    );
    let block = facts
        .blocks
        .iter()
        .find(|b| b.addr == "output.ids")
        .unwrap();
    assert!(block
        .value_refs
        .contains(&"module.network.zone_id".to_string()));
}

#[test]
fn output_value_refs_keep_module_output_suffix() {
    let facts = parse(
        r#"
        output "zone" {
          value = module.network.zone_id
        }
        "#,
    );
    let block = facts
        .blocks
        .iter()
        .find(|b| b.addr == "output.zone")
        .unwrap();
    assert!(block
        .value_refs
        .contains(&"module.network.zone_id".to_string()));
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
fn resolves_indexed_references_and_terminates_on_splat() {
    let facts = parse(
        r#"
        resource "aws_lb" "web" {
          zone    = module.net[0].zone_id
          ids     = aws_subnet.main[*].id
        }
        "#,
    );
    // Indexed module reference still resolves the module and its output.
    let module_ref = facts
        .references
        .iter()
        .find(|r| r.to_addr == "module.net")
        .expect("indexed module ref");
    assert_eq!(module_ref.module_output.as_deref(), Some("zone_id"));
    // The splat reference resolves the resource name (chain terminates at splat).
    assert!(facts
        .references
        .iter()
        .any(|r| r.to_addr == "aws_subnet.main"));
}

#[test]
fn has_extension_handles_uppercase_and_missing_extension() {
    let extensions = vec!["tf".to_string()];
    assert!(has_extension(Path::new("/repo/main.TF"), &extensions));
    assert!(!has_extension(Path::new("/repo/main.TF.JSON"), &extensions));
    assert!(!has_extension(Path::new("/repo/Makefile"), &extensions));
}

#[test]
fn ignores_incomplete_data_refs_and_non_variable_bases() {
    let facts = parse(
        r#"
        resource "aws_instance" "web" {
          partial = data.aws_ami
          wrapped = ({ inner = var.inner }).inner
        }
        "#,
    );
    let addrs = to_addrs(&facts);
    // `data.aws_ami` lacks a name, so it does not resolve to a data address.
    assert!(!addrs.iter().any(|a| a.starts_with("data.")));
    // The traversal base here is a parenthesized object, not a variable, so the
    // traversal itself yields no address — but its inner refs are still walked.
    assert!(addrs.contains(&"var.inner".to_string()));
}

#[test]
fn skips_self_references_and_recurses_nested_blocks() {
    let facts = parse(
        r#"
        resource "aws_instance" "web" {
          self_id = aws_instance.web.id
          ingress {
            cidr = var.cidr
          }
        }
        "#,
    );
    // A block referencing its own address is not recorded.
    assert!(!facts
        .references
        .iter()
        .any(|r| r.to_addr == "aws_instance.web"));
    // References inside a nested block are attributed to the enclosing resource.
    let nested = facts
        .references
        .iter()
        .find(|r| r.to_addr == "var.cidr")
        .expect("nested ref");
    assert_eq!(nested.from_addr, "aws_instance.web");
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
