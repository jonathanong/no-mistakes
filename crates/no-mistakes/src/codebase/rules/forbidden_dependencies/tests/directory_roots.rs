use super::*;

#[test]
fn directory_roots_are_recursive_filtered_and_deduplicated() {
    let root = fixture("forbidden-dependencies-directory");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|f| (f.file.as_str(), f.target.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            ("web/app/nested/page.ts", Some("fs")),
            ("web/app/page.ts", Some("fs"))
        ],
    );
    assert!(findings
        .iter()
        .all(|f| f.message.contains(&format!("dependencies '{}'", f.file))));
}

#[test]
fn literal_roots_intersect_the_prepared_file_universe() {
    let root = fixture("forbidden-dependencies-directory");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let tsconfig = resolve_tsconfig(&root, None).unwrap();
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    // Existing literal paths must not reopen files omitted from the prepared request.
    for literal in ["web/app", "web/app/page.ts"] {
        for absolute in [false, true] {
            let mut opts: Options = config.rules[0].try_rule_options().unwrap();
            opts.roots = vec![if absolute {
                root.join(literal).to_string_lossy().into_owned()
            } else {
                literal.to_string()
            }];
            for include_page in [true, false] {
                let universe = if include_page {
                    [root.join("web/app/page.ts")].into_iter().collect()
                } else {
                    crate::fx::PathSet::default()
                };
                let findings = check_rule_application(
                    &root,
                    &config,
                    &config.rules[0],
                    &opts,
                    &graph,
                    application_scope(None, Some(&universe), None),
                )
                .unwrap();
                assert_eq!(findings.len(), usize::from(include_page), "{literal}");
                if include_page {
                    assert_eq!(findings[0].file, "web/app/page.ts");
                    assert_eq!(findings[0].target.as_deref(), Some("fs"));
                }
            }
        }
    }
}

#[test]
fn directory_roots_share_prepared_facts_without_discovery_or_parsing() {
    let root = fixture("forbidden-dependencies-directory");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    super::prepared_support::assert_prepared_matches_standalone(&root, &config, 2);
}
