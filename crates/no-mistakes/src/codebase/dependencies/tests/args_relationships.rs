// Included into `dependencies::tests::args`; shares its parsing helpers.

#[test]
fn relationship_flag_parsed() {
    let a = parse(&["deps", "a.mts", "--relationship", "import"]);
    assert_eq!(a.relationships, vec![RelationshipArg::Import]);
}

#[test]
fn relationship_flag_repeatable() {
    let a = parse(&[
        "deps",
        "a.mts",
        "--relationship",
        "import",
        "--relationship",
        "test",
    ]);
    assert_eq!(a.relationships.len(), 2);
}

#[test]
fn empty_relationships_returns_standard_edges() {
    let set = relationship_filter(&[]).expect("unfiltered traversal has an explicit standard set");
    assert!(set.contains(&EdgeKind::Import));
    assert!(!set.contains(&EdgeKind::RouteImport));
}

#[test]
fn all_keyword_returns_standard_edges() {
    let set = relationship_filter(&[RelationshipArg::All])
        .expect("all excludes opt-in alternate edges");
    assert!(set.contains(&EdgeKind::Selector));
    assert!(!set.contains(&EdgeKind::RouteImport));
}

#[test]
fn standard_edges_include_every_non_opt_in_relationship_mapping() {
    use clap::ValueEnum as _;

    let standard = relationship_filter(&[]).expect("default relationship set is explicit");
    for relationship in RelationshipArg::value_variants()
        .iter()
        .copied()
        .filter(|relationship| {
            !matches!(relationship, RelationshipArg::RouteImport | RelationshipArg::All)
        })
    {
        let edges = relationship_filter(&[relationship]).expect("relationship filter is explicit");
        for edge in edges {
            assert!(
                standard.contains(&edge),
                "{relationship:?} produces {edge:?}, missing from default relationships"
            );
        }
    }
}

#[test]
fn all_and_route_import_are_repeatable_or_filters_in_either_order() {
    for relationships in [
        [RelationshipArg::All, RelationshipArg::RouteImport],
        [RelationshipArg::RouteImport, RelationshipArg::All],
    ] {
        let set = relationship_filter(&relationships).expect("combined filter is explicit");
        assert!(set.contains(&EdgeKind::Import));
        assert!(set.contains(&EdgeKind::RouteImport));
    }
}

#[test]
fn import_maps_to_all_import_forms() {
    let set = relationship_filter(&[RelationshipArg::Import]).unwrap();
    assert!(set.contains(&EdgeKind::Import));
    assert!(set.contains(&EdgeKind::TypeImport));
    assert!(set.contains(&EdgeKind::DynamicImport));
    assert!(set.contains(&EdgeKind::Require));
    assert!(!set.contains(&EdgeKind::TestOf));
}

#[test]
fn granular_imports_map_to_respective_edge_kinds() {
    let static_set = relationship_filter(&[RelationshipArg::ImportStatic]).unwrap();
    assert!(static_set.contains(&EdgeKind::Import));
    assert!(!static_set.contains(&EdgeKind::TypeImport));
    assert!(!static_set.contains(&EdgeKind::DynamicImport));
    assert!(!static_set.contains(&EdgeKind::Require));

    let dynamic_set = relationship_filter(&[RelationshipArg::ImportDynamic]).unwrap();
    assert!(!dynamic_set.contains(&EdgeKind::Import));
    assert!(!dynamic_set.contains(&EdgeKind::TypeImport));
    assert!(dynamic_set.contains(&EdgeKind::DynamicImport));
    assert!(!dynamic_set.contains(&EdgeKind::Require));

    let type_set = relationship_filter(&[RelationshipArg::ImportType]).unwrap();
    assert!(!type_set.contains(&EdgeKind::Import));
    assert!(type_set.contains(&EdgeKind::TypeImport));
    assert!(!type_set.contains(&EdgeKind::DynamicImport));
    assert!(!type_set.contains(&EdgeKind::Require));

    let require_set = relationship_filter(&[RelationshipArg::ImportRequire]).unwrap();
    assert!(!require_set.contains(&EdgeKind::Import));
    assert!(!require_set.contains(&EdgeKind::TypeImport));
    assert!(!require_set.contains(&EdgeKind::DynamicImport));
    assert!(require_set.contains(&EdgeKind::Require));
}

#[test]
fn granular_import_cli_flags_parsed() {
    let a = parse(&[
        "deps",
        "a.mts",
        "--relationship",
        "import-static",
        "--relationship",
        "import-dynamic",
        "--relationship",
        "import-type",
        "--relationship",
        "import-require",
    ]);
    assert_eq!(
        a.relationships,
        vec![
            RelationshipArg::ImportStatic,
            RelationshipArg::ImportDynamic,
            RelationshipArg::ImportType,
            RelationshipArg::ImportRequire,
        ]
    );
}

#[test]
fn workspace_maps_to_workspace_import() {
    let set = relationship_filter(&[RelationshipArg::Workspace]).unwrap();
    assert!(set.contains(&EdgeKind::WorkspaceImport));
}

#[test]
fn package_maps_to_package_dependency() {
    let set = relationship_filter(&[RelationshipArg::Package]).unwrap();
    assert!(set.contains(&EdgeKind::PackageDependency));
}

#[test]
fn test_maps_to_test_of_and_route_test() {
    let set = relationship_filter(&[RelationshipArg::Test]).unwrap();
    assert!(set.contains(&EdgeKind::TestOf));
    assert!(set.contains(&EdgeKind::RouteTest));
    assert!(set.contains(&EdgeKind::Layout));
    // Selector edges connect test files to components via data-pw; include
    // them so --relationship test covers selector-based impacted tests too.
    assert!(set.contains(&EdgeKind::Selector));
}

#[test]
fn route_maps_to_route_ref_and_route_test() {
    let set = relationship_filter(&[RelationshipArg::Route]).unwrap();
    assert!(set.contains(&EdgeKind::RouteRef));
    assert!(set.contains(&EdgeKind::RouteTest));
    assert!(set.contains(&EdgeKind::Layout));
}

#[test]
fn queue_maps_to_queue_enqueue_and_queue_worker() {
    let set = relationship_filter(&[RelationshipArg::Queue]).unwrap();
    assert!(set.contains(&EdgeKind::QueueEnqueue));
    assert!(set.contains(&EdgeKind::QueueWorker));
}

#[test]
fn md_maps_to_markdown_link() {
    let set = relationship_filter(&[RelationshipArg::Md]).unwrap();
    assert!(set.contains(&EdgeKind::MarkdownLink));
}

#[test]
fn ci_maps_to_ci_invocation() {
    let set = relationship_filter(&[RelationshipArg::Ci]).unwrap();
    assert!(set.contains(&EdgeKind::CiInvocation));
}

#[test]
fn workflow_relationships_include_their_structural_bridges() {
    let workflow = relationship_filter(&[RelationshipArg::Workflow]).unwrap();
    for edge in [
        EdgeKind::WorkflowJob,
        EdgeKind::WorkflowStep,
        EdgeKind::WorkflowNeeds,
        EdgeKind::WorkflowUses,
        EdgeKind::WorkflowRun,
        EdgeKind::WorkflowArtifact,
    ] {
        assert!(workflow.contains(&edge));
    }

    let run = relationship_filter(&[RelationshipArg::WorkflowRun]).unwrap();
    assert_eq!(
        run,
        [
            EdgeKind::WorkflowJob,
            EdgeKind::WorkflowStep,
            EdgeKind::WorkflowRun,
        ]
        .into()
    );

    let needs = relationship_filter(&[RelationshipArg::WorkflowNeeds]).unwrap();
    assert_eq!(needs, [EdgeKind::WorkflowJob, EdgeKind::WorkflowNeeds].into());

    let ci = relationship_filter(&[RelationshipArg::Ci]).unwrap();
    assert_eq!(ci, [EdgeKind::CiInvocation].into());
}

#[test]
fn multiple_kinds_combined() {
    let set = relationship_filter(&[RelationshipArg::Import, RelationshipArg::Test]).unwrap();
    assert!(set.contains(&EdgeKind::Import));
    assert!(set.contains(&EdgeKind::TestOf));
    assert!(!set.contains(&EdgeKind::QueueEnqueue));
    assert!(!set.contains(&EdgeKind::QueueWorker));
}
