use crate::check_tasks::{
    run_codebase_check_with_catalog, run_filesystem_rules_check_with_facts, run_integration_check,
    run_queue_check, run_rules_check, CodebaseCheckInputs,
};
mod inputs;
mod react_dispatch;
pub(crate) use inputs::{DomainCheckInputs, DomainResults};

pub(crate) fn run_domain_checks(inputs: DomainCheckInputs<'_>) -> DomainResults {
    let observer = no_mistakes::diagnostics::current();
    let session = inputs.session;
    let root = inputs.root;
    let config_path = inputs.config_path;
    let tsconfig_path = inputs.tsconfig_path;
    let react_enabled = inputs.react_enabled;
    let queues_enabled = inputs.queues_enabled;
    let integration_enabled = inputs.integration_enabled;
    let unique_exports_enabled = inputs.unique_exports_enabled;
    let filesystem_rules_enabled = inputs.filesystem_rules_enabled;
    let discovered_files = inputs.discovered_files;
    let facts = inputs.facts;
    let prepared_playwright = inputs.prepared_playwright;
    let prepared_react = inputs.prepared_react;
    let prepared_graph = inputs.prepared_graph;
    let dependency_graph = inputs.dependency_graph;
    let prepared_tsconfig = inputs.prepared_tsconfig;
    let prepared_tsconfig_catalog = inputs.prepared_tsconfig_catalog;
    let visible_paths = inputs.visible_paths;
    let sources = inputs.sources;
    let inferred_roots = inputs.inferred_roots;
    let config = inputs.config;
    let (codebase_config, vitest_projects) = (inputs.codebase_config, inputs.vitest_projects);
    let workflow_documents = inputs.workflow_documents;
    let tsconfig_gate_project_inputs = inputs.tsconfig_gate_project_inputs;
    let defer_suppression = inputs.defer_suppression;

    let ((react, queues), (rules, (integration, (codebase, filesystem_rules)))) = rayon::join(
        || {
            rayon::join(
                || {
                    no_mistakes::diagnostics::with_observer(observer.clone(), || {
                        react_dispatch::run(react_dispatch::Inputs {
                            root,
                            enabled: react_enabled,
                            facts,
                            prepared: prepared_react,
                            sources: sources.as_ref(),
                            defer_suppression,
                        })
                    })
                },
                || {
                    no_mistakes::diagnostics::with_observer(observer.clone(), || {
                        run_queue_check(
                            root,
                            prepared_tsconfig_catalog,
                            queues_enabled,
                            facts,
                            &session,
                        )
                    })
                },
            )
        },
        || {
            rayon::join(
                || {
                    no_mistakes::diagnostics::with_observer(observer.clone(), || {
                        run_rules_check(
                            no_mistakes::codebase::rules::PreparedRulesCheck {
                                session: session.clone(),
                                root,
                                config_path: config_path.as_deref(),
                                tsconfig_path: tsconfig_path.as_deref(),
                                shared: facts,
                                prepared_playwright,
                                config,
                                prepared_graph,
                                prepared_tsconfig,
                                prepared_tsconfig_catalog,
                                inferred_roots: Some(inferred_roots),
                                sources: Some(sources.as_ref()),
                            },
                            dependency_graph.as_deref(),
                            sources.as_ref(),
                            defer_suppression,
                        )
                    })
                },
                || {
                    rayon::join(
                        || {
                            no_mistakes::diagnostics::with_observer(observer.clone(), || {
                                run_integration_check(
                                    &session,
                                    root,
                                    integration_enabled,
                                    config,
                                    facts,
                                    prepared_tsconfig_catalog,
                                    visible_paths,
                                )
                            })
                        },
                        || {
                            rayon::join(
                                || {
                                    no_mistakes::diagnostics::with_observer(
                                        observer.clone(),
                                        || {
                                            run_codebase_check_with_catalog(CodebaseCheckInputs {
                                                session: &session,
                                                root,
                                                config: codebase_config,
                                                prepared_tsconfig_catalog,
                                                enabled: unique_exports_enabled,
                                                facts,
                                                inferred_roots,
                                                defer_suppression,
                                            })
                                        },
                                    )
                                },
                                || {
                                    no_mistakes::diagnostics::with_observer(
                                        observer.clone(),
                                        || {
                                            run_filesystem_rules_check_with_facts(
                                                root,
                                                config,
                                                filesystem_rules_enabled,
                                                discovered_files,
                                                no_mistakes::codebase::rules::filesystem_dispatch::PreparedFilesystemRuleInputs {
                                                    snapshot: visible_paths,
                                                    sources: std::sync::Arc::clone(&sources),
                                                    vitest_catalog: vitest_projects,
                                                    workflow_documents,
                                                    tsconfig_gate_project_inputs,
                                                    config_path: config_path.as_deref(),
                                                },
                                                Some(facts),
                                                defer_suppression,
                                            )
                                        },
                                    )
                                },
                            )
                        },
                    )
                },
            )
        },
    );
    (
        react,
        queues,
        rules,
        integration,
        codebase,
        filesystem_rules,
    )
}
