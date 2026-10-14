use super::provenance::saved_graph;
use super::*;

#[test]
fn direct_transaction_and_helper_query_keep_separate_physical_evidence() {
    use crate::codebase::dependencies::graph::{
        KnownCallTarget, ResolvedCallAlternatives, ResolvedCallCandidate, ResolvedCallTarget,
    };
    let fixture = alternatives_fixture();
    let graph = saved_graph(fixture.path());
    let config: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(
        &std::fs::read_to_string(fixture.path().join(".no-mistakes.yml")).unwrap(),
    )
    .unwrap();
    let selection = selection_from_config(&config, "database", &[]).unwrap();
    let leaf = |export: &str| ResolvedCallCandidate {
        target: KnownCallTarget::ModuleExport {
            specifier: "@vendor/database".into(),
            export_path: export.into(),
            repository_target: None,
            callable_id: None,
        },
        node: None,
    };
    let helper = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.source_callee == "queryHelper")
        .unwrap();
    let helper_leaf = ResolvedCallCandidate {
        target: match &helper.target {
            ResolvedCallTarget::RepositoryFunction { file, scope } => {
                KnownCallTarget::RepositoryFunction {
                    file: file.clone(),
                    scope: scope.clone(),
                }
            }
            target => panic!("saved helper must be repository-owned: {target:?}"),
        },
        node: helper.target_node.clone(),
    };
    let transaction = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.source_callee == "begin")
        .unwrap();
    let query = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.source_callee == "primary.read" && site.file.ends_with("provenance.mts"))
        .unwrap();
    for known in [
        vec![leaf("begin"), helper_leaf.clone()],
        vec![helper_leaf, leaf("begin")],
    ] {
        let mut site = transaction.clone();
        site.target = ResolvedCallTarget::Alternatives(std::sync::Arc::new(
            ResolvedCallAlternatives::new(known, false),
        ));
        let mut direct = Vec::new();
        visit_effect_matches(&selection, &site, &crate::fx::fx_map(), &mut |candidate| {
            direct.push(candidate)
        });
        assert_eq!(
            direct,
            [effect_alternative(&config.effects["database"].targets[3])]
        );
        let mut reached = Vec::new();
        visit_effect_matches(&selection, query, &crate::fx::fx_map(), &mut |candidate| {
            reached.push(candidate)
        });
        assert_eq!(
            reached
                .iter()
                .map(|candidate| candidate.category.as_deref())
                .collect::<Vec<_>>(),
            [Some("read")]
        );
        assert_eq!(site.offset, transaction.offset);
    }
}

#[test]
fn runtime_alternative_rows_keep_the_saved_physical_offset() {
    let fixture = alternatives_fixture();
    let graph = saved_graph(fixture.path());
    let source = std::fs::read_to_string(fixture.path().join("src/entry.mts")).unwrap();
    let sites = graph.call_sites_in_file(&fixture.path().join("src/entry.mts"));
    let physical = sites
        .iter()
        .filter(|site| site.source_callee == "receiver.read")
        .collect::<Vec<_>>();
    assert_eq!(physical.len(), 1);
    assert_eq!(
        physical[0].offset as usize,
        source.find("receiver.read()").unwrap()
    );
    assert_eq!(physical[0].targets().len(), 2);
}
