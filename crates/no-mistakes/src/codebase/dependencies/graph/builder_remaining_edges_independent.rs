struct IndependentRemainingEdges {
    markdown: Vec<Edge>,
    terraform: Vec<Edge>,
    dotnet: Vec<Edge>,
    swift: Vec<Edge>,
}

/// Markdown, Terraform, .NET, and Swift only read prepared inputs. Collect
/// their edges in parallel and merge on the caller thread.
fn collect_independent_remaining_edges(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> IndependentRemainingEdges {
    let (markdown, (terraform, (dotnet, swift))) = rayon::join(
        || {
            crate::perf_trace::trace("graph.markdown", || {
                edge_inputs
                    .plan
                    .markdown
                    .then(|| {
                        collect_md_edges(&edge_inputs.graph_files.all, edge_inputs.graph_files)
                    })
                    .unwrap_or_default()
            })
        },
        || {
            rayon::join(
                || {
                    crate::perf_trace::trace("graph.terraform", || {
                        collect_terraform_edges_for_plan(edge_inputs)
                    })
                },
                || {
                    rayon::join(
                        || {
                            crate::perf_trace::trace("graph.dotnet", || {
                                collect_dotnet_edges_for_plan(edge_inputs)
                            })
                        },
                        || {
                            crate::perf_trace::trace("graph.swift", || {
                                collect_swift_edges_for_plan(edge_inputs, facts, session)
                            })
                        },
                    )
                },
            )
        },
    );
    IndependentRemainingEdges {
        markdown,
        terraform,
        dotnet,
        swift,
    }
}

fn merge_independent_remaining_edges(
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
    edges: IndependentRemainingEdges,
) {
    merge_edges(forward, reverse, edges.markdown);
    merge_seeded_edges(forward, reverse, edges.terraform);
    merge_seeded_edges(forward, reverse, edges.dotnet);
    merge_seeded_edges(forward, reverse, edges.swift);
}
