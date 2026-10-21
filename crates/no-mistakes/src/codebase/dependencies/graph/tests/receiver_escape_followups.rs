use super::receiver_escapes::{assert_export, assert_unknown, graph};

fn unknown_replica(file: &str) {
    let (built, root) = graph();
    assert_unknown(&built, &root, file, "replica.get");
    assert_export(&built, &root, file, "unrelated.get");
}

fn unknown_replica_keeps_primary(file: &str) {
    let (built, root) = graph();
    assert_unknown(&built, &root, file, "replica.get");
    assert_export(&built, &root, file, "primary.get");
    assert_export(&built, &root, file, "unrelated.get");
}

#[test]
fn class_method_this_rebind_reaches_the_argument() {
    unknown_replica("class-method-this");
}

#[test]
fn subclass_method_reaches_the_inherited_parameter() {
    unknown_replica("class-method-subclass");
}

#[test]
fn const_instance_alias_keeps_the_class() {
    unknown_replica("class-method-alias");
}

#[test]
fn instance_stored_on_an_object_keeps_the_class() {
    unknown_replica("class-method-holder");
}

#[test]
fn class_field_arrow_is_a_method_provider() {
    unknown_replica("class-method-field");
}

#[test]
fn factory_return_is_an_instance() {
    unknown_replica("class-method-factory");
}

#[test]
fn computed_method_uses_the_literal_name() {
    unknown_replica("class-method-computed");
}

#[test]
fn method_alias_call_shifts_off_the_this_argument() {
    unknown_replica("class-method-call");
}

#[test]
fn member_call_shifts_off_the_this_argument() {
    unknown_replica("class-method-member-call");
}

#[test]
fn logical_and_keeps_the_left_receiver() {
    unknown_replica("logical-and-left");
}

#[test]
fn logical_or_keeps_the_left_receiver() {
    unknown_replica("logical-or-left");
}

#[test]
fn nullish_coalesce_keeps_the_left_receiver() {
    unknown_replica("logical-nullish-left");
}

#[test]
fn logical_or_assign_includes_the_right_hand_receiver() {
    unknown_replica_keeps_primary("logical-or-assign");
}

#[test]
fn logical_and_assign_includes_the_right_hand_receiver() {
    unknown_replica_keeps_primary("logical-and-assign");
}

#[test]
fn reduce_right_element_is_the_second_parameter() {
    unknown_replica("reduce-right");
}

#[test]
fn push_before_a_spread_is_an_element() {
    unknown_replica("array-push-spread");
}

#[test]
fn push_before_for_each_is_an_element() {
    unknown_replica("array-push-foreach");
}

#[test]
fn spread_uses_the_array_before_a_later_reassignment() {
    let (built, root) = graph();
    assert_unknown(&built, &root, "array-reassign-after", "replica.get");
    assert_export(&built, &root, "array-reassign-after", "untouched.get");
    assert_export(&built, &root, "array-reassign-after", "unrelated.get");
}

#[test]
fn spread_uses_the_array_after_an_earlier_reassignment() {
    let (built, root) = graph();
    assert_export(&built, &root, "array-reassign-before", "replica.get");
    assert_unknown(&built, &root, "array-reassign-before", "untouched.get");
    assert_export(&built, &root, "array-reassign-before", "unrelated.get");
}

#[test]
fn object_assign_spread_sees_a_pushed_source() {
    // Replacing `client` and then writing `.get` may also drop `primary.get`.
    unknown_replica("object-assign-array-push");
}

#[test]
fn named_array_conditional_element_reaches_both_arms() {
    unknown_replica("named-conditional-spread");
}

#[test]
fn for_each_conditional_element_reaches_both_arms() {
    unknown_replica("foreach-conditional");
}

#[test]
fn higher_order_apply_passes_the_receiver() {
    unknown_replica_keeps_primary("higher-order-apply");
}

#[test]
fn arguments_alias_reaches_the_call_argument() {
    unknown_replica_keeps_primary("arguments-alias");
}

#[test]
fn arrow_arguments_belong_to_the_enclosing_function() {
    let (built, root) = graph();
    assert_unknown(&built, &root, "arrow-arguments", "replica.get");
    assert_export(&built, &root, "arrow-arguments", "untouched.get");
    assert_export(&built, &root, "arrow-arguments", "unrelated.get");
}

#[test]
fn callback_array_parameter_reads_the_element() {
    unknown_replica_keeps_primary("callback-array-slot");
}

fn guarded_array_keeps_both_origins(file: &str, newly_recovered: bool) {
    let (built, root) = graph();
    if newly_recovered {
        assert_possible_export(&built, &root, file, "replica.get");
    } else {
        assert_unknown(&built, &root, file, "replica.get");
    }
    assert_unknown(&built, &root, file, "untouched.get");
    for callee in ["primary.get", "replica.info", "unrelated.get"] {
        assert_export(&built, &root, file, callee);
    }
}

#[test]
fn conditional_array_reassignment_keeps_the_earlier_and_later_origins() {
    guarded_array_keeps_both_origins("array-reassign-conditional", true);
}

#[test]
fn loop_array_reassignment_keeps_the_earlier_and_later_origins() {
    guarded_array_keeps_both_origins("array-reassign-loop", true);
}

#[test]
fn conditional_array_push_keeps_the_initial_and_appended_origins() {
    guarded_array_keeps_both_origins("array-push-conditional", false);
}

#[test]
fn loop_array_push_keeps_the_initial_and_appended_origins() {
    guarded_array_keeps_both_origins("array-push-loop", false);
}

#[test]
fn super_method_rebind_uses_the_base_instead_of_the_override() {
    let (built, root) = graph();
    assert_possible_export(&built, &root, "class-method-super", "replica.get");
    for callee in ["primary.info", "replica.info", "unrelated.get"] {
        assert_export(&built, &root, "class-method-super", callee);
    }
}

fn assert_possible_export(
    built: &super::DepGraph,
    root: &std::path::Path,
    file: &str,
    callee: &str,
) {
    let site = built
        .call_sites_in_file(&root.join(format!("src/{file}.mts")))
        .iter()
        .find(|site| site.source_callee == callee)
        .unwrap();
    assert!(
        matches!(&site.target, super::ResolvedCallTarget::Alternatives(_)),
        "{site:?}"
    );
    assert!(site.has_unknown_target(), "{site:?}");
    let candidates = site.targets().collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1, "{site:?}");
    assert!(
        matches!(candidates[0].target, super::KnownCallTargetRef::ModuleExport { export_path, .. }
        if export_path == callee),
        "{site:?}"
    );
}

#[test]
fn guarded_array_effects_preserve_the_baseline_call_sites() {
    let root = super::fixture("value-flow-receiver-escapes");
    let report = crate::effects_query::run(
        &root,
        Some(&root.join("effects.yml")),
        None,
        "receiver",
        std::path::Path::new("src/array-reassign-conditional.mts"),
        &[],
        None,
    )
    .unwrap();
    // These four matches were reported before the guarded origin was recovered.
    // Widening that target to possible+unknown must not silence an existing effect.
    let observed = report
        .call_sites
        .iter()
        .map(|site| (site.file.as_str(), site.line, site.callee.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        observed,
        [
            ("src/array-reassign-conditional.mts", 12, "replica.get"),
            ("src/array-reassign-conditional.mts", 14, "primary.get"),
            ("src/array-reassign-conditional.mts", 15, "replica.info"),
            ("src/array-reassign-conditional.mts", 16, "unrelated.get"),
        ]
    );
}

#[test]
fn super_uses_the_class_declaration_base_despite_a_method_local_shadow() {
    let (built, root) = graph();
    assert_possible_export(&built, &root, "class-method-super-shadow", "replica.get");
    for callee in ["primary.info", "replica.info", "unrelated.get"] {
        assert_export(&built, &root, "class-method-super-shadow", callee);
    }
}

#[test]
fn unconditional_array_reset_discards_earlier_guarded_origins() {
    let (built, root) = graph();
    assert_export(
        &built,
        &root,
        "array-reassign-conditional-reset",
        "replica.get",
    );
    assert_unknown(
        &built,
        &root,
        "array-reassign-conditional-reset",
        "untouched.get",
    );
    assert_export(
        &built,
        &root,
        "array-reassign-conditional-reset",
        "unrelated.get",
    );
}

#[test]
fn guarded_array_lengths_preserve_origins_of_shifted_arguments() {
    guarded_array_keeps_both_origins("array-reassign-conditional-shift", true);
}

#[test]
fn guarded_escape_preserves_a_repository_method_candidate() {
    let (built, root) = graph();
    let site = built
        .call_sites_in_file(&root.join("src/array-guarded-local-method.mts"))
        .iter()
        .find(|site| site.source_callee == "api.get")
        .unwrap();
    assert!(site.has_unknown_target(), "{site:?}");
    assert!(
        site.targets().any(|candidate| matches!(
            candidate.target,
            super::KnownCallTargetRef::RepositoryFunction {
                scope: "original",
                ..
            }
        )),
        "{site:?}"
    );
    assert_export(&built, &root, "array-guarded-local-method", "unrelated.get");
}

#[test]
fn guarded_factory_keeps_returned_executor_and_factory_provenance() {
    let (built, root) = graph();
    let site = built
        .call_sites_in_file(&root.join("src/array-guarded-factory-return.mts"))
        .iter()
        .find(|site| site.source_callee == "made")
        .unwrap();
    assert!(site.has_unknown_target(), "{site:?}");
    assert!(
        site.targets().any(|candidate| matches!(
            candidate.target,
            super::KnownCallTargetRef::ModuleExport {
                export_path: "primary.get",
                ..
            }
        )),
        "{site:?}"
    );
    assert!(
        site.returned_from.iter().any(|(module, export)| module
            == "./uncertain-factory-provider.mts"
            && export == "make"),
        "{site:?}"
    );
    assert_export(
        &built,
        &root,
        "array-guarded-factory-return",
        "unrelated.get",
    );
}

#[test]
fn guarded_escape_keeps_existing_authored_choice_metadata() {
    let (built, root) = graph();
    let site = built
        .call_sites_in_file(&root.join("src/array-guarded-existing-choice.mts"))
        .iter()
        .find(|site| site.source_callee == "api.get")
        .unwrap();
    assert!(site.has_unknown_target(), "{site:?}");
    assert!(
        site.targets().any(|candidate| matches!(
            candidate.target,
            super::KnownCallTargetRef::ModuleExport {
                export_path: "primary.get",
                ..
            }
        )),
        "{site:?}"
    );
    assert_eq!(
        site.alternatives.len(),
        2,
        "only the authored nullish alternatives survive"
    );
    assert_export(
        &built,
        &root,
        "array-guarded-existing-choice",
        "unrelated.get",
    );
}

#[test]
fn guarded_multiple_spreads_keep_const_tail_origins_and_positions() {
    guarded_array_keeps_both_origins("array-guarded-multiple-spreads", true);
}

#[test]
fn guarded_method_copy_keeps_the_original_replacement_payload_candidate() {
    let (built, root) = graph();
    let site = built
        .call_sites_in_file(&root.join("src/array-guarded-replacement-copy.mts"))
        .iter()
        .find(|site| site.source_callee == "destination.get")
        .unwrap();
    assert!(site.has_unknown_target(), "{site:?}");
    assert!(
        site.targets().any(|candidate| matches!(
            candidate.target,
            super::KnownCallTargetRef::ModuleExport {
                export_path: "replica.get",
                ..
            }
        )),
        "{site:?}"
    );
    assert_export(
        &built,
        &root,
        "array-guarded-replacement-copy",
        "primary.get",
    );
    assert_export(
        &built,
        &root,
        "array-guarded-replacement-copy",
        "unrelated.get",
    );
}

#[test]
fn guarded_inline_factory_keeps_the_returned_executor_candidate() {
    let (built, root) = graph();
    let sites = built.call_sites_in_file(&root.join("src/array-guarded-inline-factory.mts"));
    let site = sites
        .iter()
        .find(|site| {
            site.returned_from
                .iter()
                .any(|(_, export)| export == "make")
        })
        .unwrap();
    assert!(site.has_unknown_target(), "{site:?}");
    assert!(
        site.targets().any(|candidate| matches!(
            candidate.target,
            super::KnownCallTargetRef::ModuleExport {
                export_path: "primary.get",
                ..
            }
        )),
        "{site:?}"
    );
    let invocation = sites
        .iter()
        .find(|site| site.source_callee == "value")
        .unwrap();
    assert!(
        invocation.targets().any(|candidate| matches!(
            candidate.target,
            super::KnownCallTargetRef::ModuleExport {
                export_path: "primary.get",
                ..
            }
        )),
        "{invocation:?}"
    );
    assert_export(
        &built,
        &root,
        "array-guarded-inline-factory",
        "unrelated.get",
    );
}

#[test]
fn earlier_entries_of_later_spreads_follow_prior_open_shifts() {
    guarded_array_keeps_both_origins("array-guarded-open-earlier-entry", true);
}

#[test]
fn possible_foreign_forward_replay_does_not_become_definite() {
    guarded_array_keeps_both_origins("array-guarded-foreign-forward", true);
}

#[test]
fn retained_possible_foreign_writes_keep_later_providers_possible() {
    let (built, root) = graph();
    for callee in ["primary.get", "replica.get"] {
        assert_possible_export(&built, &root, "array-guarded-foreign-provider", callee);
    }
    assert_unknown(
        &built,
        &root,
        "array-guarded-foreign-provider",
        "untouched.get",
    );
    for callee in ["replica.info", "unrelated.get"] {
        assert_export(&built, &root, "array-guarded-foreign-provider", callee);
    }
}

#[path = "receiver_escape_union.rs"]
mod union;
