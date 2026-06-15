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
