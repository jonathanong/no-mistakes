use super::*;

#[test]
fn terraform_edge_collector_covers_empty_config_branches() {
    let root = fixture("terraform-basic");
    let all_files = GraphFiles::discover(&root).all;

    // No config options at all.
    assert!(collect_terraform_edges(&root, &all_files, None).is_empty());

    // Configured options but no module roots.
    let mut options = graph_config_options(&root).expect("terraform fixture config should parse");
    options.terraform.module_roots.clear();
    assert!(collect_terraform_edges(&root, &all_files, Some(&options)).is_empty());

    // Module roots configured but no files supplied.
    let options = graph_config_options(&root).expect("terraform fixture config should parse");
    assert!(collect_terraform_edges(&root, &[], Some(&options)).is_empty());
}

#[test]
fn terraform_edges_emit_reference_module_and_output_kinds() {
    let root = fixture("terraform-basic");
    let all_files = GraphFiles::discover(&root).all;
    let options = graph_config_options(&root).expect("terraform fixture config should parse");

    let edges = collect_terraform_edges(&root, &all_files, Some(&options));

    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::TerraformReference));
    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::TerraformModuleRef));
    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::TerraformOutputRef));
}

#[test]
fn terraform_bare_module_reference_links_module_files() {
    use crate::codebase::terraform::{TerraformFactMap, TerraformFileFacts, TerraformRef};
    use std::collections::BTreeSet;

    // `infra/prod/main.tf` has `depends_on = [module.net]`; the module source
    // lives in `infra/net`. A bare module reference should link to its files.
    let consumer = PathBuf::from("/repo/infra/prod/main.tf");
    let module_file = PathBuf::from("/repo/infra/net/main.tf");
    let mut facts = TerraformFactMap::default();
    facts.files.insert(
        module_file.clone(),
        TerraformFileFacts {
            path: module_file.clone(),
            module_dir: PathBuf::from("/repo/infra/net"),
            blocks: Vec::new(),
            references: Vec::new(),
        },
    );
    facts
        .module_sources
        .insert("module.net".to_string(), PathBuf::from("/repo/infra/net"));
    facts
        .files_by_module
        .insert(PathBuf::from("/repo/infra/net"), BTreeSet::from([module_file.clone()]));
    facts.refs_to.insert(
        "module.net".to_string(),
        vec![TerraformRef {
            from_file: consumer.clone(),
            from_addr: "aws_lb.web".to_string(),
            to_addr: "module.net".to_string(),
            module_output: None,
        }],
    );

    let mut edges = Vec::new();
    collect_terraform_output_edges(&facts, &mut edges);
    assert!(edges.iter().any(|(from, to, kind)| from == &NodeId::File(consumer.clone())
        && to == &NodeId::File(module_file.clone())
        && *kind == EdgeKind::TerraformModuleRef));
}

#[test]
fn terraform_reference_edges_stay_within_a_module() {
    use crate::codebase::terraform::{
        TerraformBlock, TerraformFactMap, TerraformFileFacts, TerraformRef, TfBlockKind,
    };
    use std::collections::BTreeSet;

    // Two modules each declare `aws_s3_bucket.main`; m1/use.tf references it.
    let decl1 = PathBuf::from("/repo/m1/main.tf");
    let use1 = PathBuf::from("/repo/m1/use.tf");
    let decl2 = PathBuf::from("/repo/m2/main.tf");
    let mut facts = TerraformFactMap::default();
    for (path, dir) in [(&decl1, "/repo/m1"), (&use1, "/repo/m1"), (&decl2, "/repo/m2")] {
        facts.files.insert(
            path.clone(),
            TerraformFileFacts {
                path: path.clone(),
                module_dir: PathBuf::from(dir),
                blocks: vec![TerraformBlock {
                    kind: TfBlockKind::Resource,
                    addr: "aws_s3_bucket.main".to_string(),
                    name: "main".to_string(),
                    file: path.clone(),
                    module_source_dir: None,
                    value_refs: Vec::new(),
                }],
                references: Vec::new(),
            },
        );
    }
    facts
        .declarations
        .insert("aws_s3_bucket.main".to_string(), BTreeSet::from([decl1.clone(), decl2.clone()]));
    facts.refs_to.insert(
        "aws_s3_bucket.main".to_string(),
        vec![TerraformRef {
            from_file: use1.clone(),
            from_addr: "aws_lb.web".to_string(),
            to_addr: "aws_s3_bucket.main".to_string(),
            module_output: None,
        }],
    );

    let mut edges = Vec::new();
    collect_terraform_reference_edges(&facts, &mut edges);

    // The reference resolves only to the same-module declaration, never m2's.
    assert!(edges
        .iter()
        .any(|(from, to, _)| from == &NodeId::File(use1.clone()) && to == &NodeId::File(decl1.clone())));
    assert!(!edges
        .iter()
        .any(|(_, to, _)| to == &NodeId::File(decl2.clone())));
}

#[test]
fn terraform_edge_collectors_handle_missing_lookups() {
    use crate::codebase::terraform::{
        TerraformBlock, TerraformFactMap, TerraformFileFacts, TerraformRef, TfBlockKind,
    };

    let mut facts = TerraformFactMap::default();
    // A module block with no resolved source dir, and one pointing at a dir with
    // no known files — both should produce no module edges.
    let file = PathBuf::from("/repo/m/main.tf");
    facts.files.insert(
        file.clone(),
        TerraformFileFacts {
            path: file.clone(),
            module_dir: PathBuf::from("/repo/m"),
            blocks: vec![
                TerraformBlock {
                    kind: TfBlockKind::Module,
                    addr: "module.a".to_string(),
                    name: "a".to_string(),
                    file: file.clone(),
                    module_source_dir: None,
                    value_refs: Vec::new(),
                },
                TerraformBlock {
                    kind: TfBlockKind::Module,
                    addr: "module.b".to_string(),
                    name: "b".to_string(),
                    file: file.clone(),
                    module_source_dir: Some(PathBuf::from("/repo/missing")),
                    value_refs: Vec::new(),
                },
            ],
            references: Vec::new(),
        },
    );
    // A reference from an unknown file (no module), and a reference from a known
    // file to an address that is never declared.
    facts.refs_to.insert(
        "aws_x.y".to_string(),
        vec![
            TerraformRef {
                from_file: PathBuf::from("/repo/unknown.tf"),
                from_addr: "aws_a.b".to_string(),
                to_addr: "aws_x.y".to_string(),
                module_output: None,
            },
            TerraformRef {
                from_file: file.clone(),
                from_addr: "aws_a.b".to_string(),
                to_addr: "aws_x.y".to_string(),
                module_output: None,
            },
        ],
    );
    // An output reference whose module is not in `module_sources`.
    facts.refs_to.insert(
        "module.ghost".to_string(),
        vec![TerraformRef {
            from_file: file.clone(),
            from_addr: "aws_a.b".to_string(),
            to_addr: "module.ghost".to_string(),
            module_output: Some("out".to_string()),
        }],
    );
    // An output reference whose module resolves, but the output is not declared.
    facts
        .module_sources
        .insert("module.real".to_string(), PathBuf::from("/repo/sub"));
    facts.refs_to.insert(
        "module.real".to_string(),
        vec![TerraformRef {
            from_file: file.clone(),
            from_addr: "aws_a.b".to_string(),
            to_addr: "module.real".to_string(),
            module_output: Some("undeclared".to_string()),
        }],
    );

    let mut edges = Vec::new();
    collect_terraform_reference_edges(&facts, &mut edges);
    collect_terraform_module_edges(&facts, &mut edges);
    collect_terraform_output_edges(&facts, &mut edges);
    assert!(edges.is_empty());
}

#[test]
fn terraform_edges_surface_in_full_graph_dependents() {
    let root = fixture("terraform-basic");
    let tsconfig = TsConfig::default();
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::all()).unwrap();

    // aws_route53_record.foo (in main.tf) is referenced by aws_lb.web (main.tf)
    // and output.record_id (outputs.tf); its dependents include outputs.tf.
    let main_tf = NodeId::File(root.join("infra/envs/prod/main.tf"));
    let dependents = graph
        .dependents_of(&[main_tf], None, None)
        .into_iter()
        .map(|entry| entry.node)
        .collect::<Vec<_>>();
    assert!(dependents
        .iter()
        .any(|node| matches!(node, NodeId::File(path) if path.ends_with("outputs.tf"))));
}
