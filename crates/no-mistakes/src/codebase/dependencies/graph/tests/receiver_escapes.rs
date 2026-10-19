use super::*;

pub(super) fn graph() -> (DepGraph, std::path::PathBuf) {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("value-flow-receiver-escapes"));
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    (graph, root)
}

fn site<'a>(
    graph: &'a DepGraph,
    root: &std::path::Path,
    file: &str,
    callee: &str,
) -> &'a ResolvedCallSite {
    graph
        .call_sites_in_file(&root.join(format!("src/{file}.mts")))
        .iter()
        .find(|site| site.source_callee == callee)
        .unwrap_or_else(|| panic!("{file} lacks {callee}"))
}

pub(super) fn assert_unknown(graph: &DepGraph, root: &std::path::Path, file: &str, callee: &str) {
    let site = site(graph, root, file, callee);
    assert!(
        site.has_unknown_target() && site.targets().next().is_none(),
        "{file} {callee}: {site:?}"
    );
}

pub(super) fn assert_export(graph: &DepGraph, root: &std::path::Path, file: &str, callee: &str) {
    let site = site(graph, root, file, callee);
    assert!(
        matches!(&site.target, ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == callee),
        "{file} {callee}: {site:?}"
    );
}

fn assert_mutated_receiver(file: &str) {
    let (graph, root) = graph();
    assert_unknown(&graph, &root, file, "replica.get");
    assert_export(&graph, &root, file, "primary.get");
}

#[test]
fn class_method_rebind_does_not_keep_a_known_client() {
    let (graph, root) = graph();
    assert_unknown(&graph, &root, "class-method", "replica.get");
    // Replacing `client` and then writing `.get` may also drop the previous
    // client. Sibling exports that were not written stay known.
    for callee in ["primary.info", "replica.info", "unrelated.get"] {
        assert_export(&graph, &root, "class-method", callee);
    }
}

#[test]
fn stored_class_method_rebind_does_not_keep_a_known_client() {
    let (graph, root) = graph();
    assert_unknown(&graph, &root, "class-method-stored", "replica.get");
    assert_export(&graph, &root, "class-method-stored", "unrelated.get");
}

#[test]
fn wrapper_write_reaches_the_outer_argument() {
    assert_mutated_receiver("wrapper-write");
}

#[test]
fn map_callback_sees_the_literal_element() {
    assert_mutated_receiver("map-mutate");
}

#[test]
fn named_array_for_each_sees_the_element() {
    assert_mutated_receiver("named-foreach");
}

#[test]
fn named_array_for_of_sees_the_element() {
    assert_mutated_receiver("named-for-of");
}

#[test]
fn conditional_argument_invalidates_the_rebound_receiver() {
    assert_mutated_receiver("conditional-arg");
}

#[test]
fn spread_argument_invalidates_the_element() {
    assert_mutated_receiver("spread-arg");
}

#[test]
fn arguments_index_write_reaches_the_call_argument() {
    assert_mutated_receiver("arguments-write");
}

#[test]
fn reduce_initial_value_is_the_accumulator_receiver() {
    assert_mutated_receiver("reduce-accumulator");
}

#[test]
fn reassigned_parameter_includes_the_live_binding() {
    assert_mutated_receiver("reassigned-parameter");
}

#[test]
fn logical_reassignment_includes_the_nullish_fallback() {
    let (graph, root) = graph();
    assert_unknown(&graph, &root, "logical-reassign", "replica.get");
    assert_export(&graph, &root, "logical-reassign", "primary.get");
    assert_export(&graph, &root, "logical-reassign", "unrelated.get");
}

#[test]
fn object_assign_spread_source_is_a_provider() {
    let (graph, root) = graph();
    assert_unknown(&graph, &root, "object-assign-spread", "replica.get");
    assert_export(&graph, &root, "object-assign-spread", "unrelated.get");
}

#[test]
fn object_assign_overwrites_the_member_method() {
    let (graph, root) = graph();
    assert_unknown(&graph, &root, "object-assign-member", "replica.get");
    assert_export(&graph, &root, "object-assign-member", "primary.get");
    assert_export(&graph, &root, "object-assign-member", "unrelated.get");
}

#[test]
fn fresh_object_assign_binding_sees_the_copied_client() {
    let (graph, root) = graph();
    assert_unknown(&graph, &root, "object-assign-fresh", "replica.get");
    assert_export(&graph, &root, "object-assign-fresh", "primary.get");
    assert_export(&graph, &root, "object-assign-fresh", "unrelated.get");
}

#[test]
fn object_assign_field_read_is_unknown_not_the_previous_or_copied_callee() {
    let (graph, root) = graph();
    assert_unknown(&graph, &root, "object-assign-read", "box.client.get");
    assert_export(&graph, &root, "object-assign-read", "primary.get");
    assert_export(&graph, &root, "object-assign-read", "replica.get");
    assert_export(&graph, &root, "object-assign-read", "unrelated.get");
    assert_unknown(&graph, &root, "direct-assign-read", "box.client.get");
    assert_export(&graph, &root, "direct-assign-read", "primary.get");
    assert_export(&graph, &root, "direct-assign-read", "replica.get");
}
