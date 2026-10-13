use super::*;

#[test]
fn nested_list_items_do_not_shift_step_locations() {
    let findings = run("nested", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 13);
    assert!(run("nested-suppressed", "{}").is_empty());
}

#[test]
fn indentless_step_sequence_has_usable_locations() {
    let findings = run("indentless", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 8);
    assert!(run("indentless-suppressed", "{}").is_empty());
}

#[test]
fn parallel_group_checks_nested_executable_steps() {
    let findings = run("parallel", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 7);
    assert_eq!(
        findings[0].target.as_deref(),
        Some(".github/workflows/ci.yml#build#1/frontend")
    );
    assert!(run("parallel-suppressed", "{}").is_empty());
}

#[test]
fn multiline_parallel_name_keeps_child_step_location() {
    let findings = run("multiline-parallel", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 10);
    assert!(run("multiline-parallel-suppressed", "{}").is_empty());
}

#[test]
fn nested_step_allow_uses_group_and_child_selectors() {
    let findings = run(
        "parallel",
        r#"
allow:
  - step: ".github/workflows/ci.yml#build#1/frontend"
    reason: "Covered by an external deadline."
"#,
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn job_key_comments_keep_step_locations_and_suppression() {
    let findings = run("commented-job", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 6);
    assert!(run("commented-job-suppressed", "{}").is_empty());
}

#[test]
fn workflow_dispatch_input_named_like_job_does_not_shift_step_location() {
    let findings = run("duplicate-job-name", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 11);
    assert!(run("duplicate-job-name-suppressed", "{}").is_empty());
}

#[test]
fn anchored_job_has_usable_step_location() {
    let findings = run("anchored-job", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 6);
    assert!(run("anchored-job-suppressed", "{}").is_empty());
}

#[test]
fn aliased_job_reports_inherited_steps_on_alias_line() {
    let findings = run("aliased-job", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 8);
    assert!(run("aliased-job-suppressed", "{}").is_empty());
}

#[test]
fn quoted_and_anchored_mapping_keys_keep_nested_locations() {
    let findings = run("decorated-keys", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 11);
    assert!(run("decorated-keys-suppressed", "{}").is_empty());
}

#[test]
fn flow_style_steps_use_their_declaration_line() {
    let findings = run("flow-steps", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 5);
    assert!(run("flow-steps-suppressed", "{}").is_empty());
}

#[test]
fn anchored_flow_style_steps_use_declaration_line() {
    let findings = run("anchored-flow-steps", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 5);
    assert!(run("anchored-flow-steps-suppressed", "{}").is_empty());
}

#[test]
fn flow_style_parallel_and_jobs_use_declaration_lines() {
    for (scenario, line) in [("flow-parallel", 6), ("flow-jobs", 2)] {
        let findings = run(scenario, "{}");
        assert_eq!(findings.len(), 1, "{scenario}: {findings:?}");
        assert_eq!(findings[0].line, line);
        assert!(run(&format!("{scenario}-suppressed"), "{}").is_empty());
    }
}

#[test]
fn flow_step_mapping_with_parallel_uses_group_line() {
    let findings = run("flow-step-mapping", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 6);
    assert!(run("flow-step-mapping-suppressed", "{}").is_empty());
}

#[test]
fn name_alias_does_not_hide_block_parallel_child_line() {
    let findings = run("alias-name-parallel", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 8);
    assert!(run("alias-name-parallel-suppressed", "{}").is_empty());
}

#[test]
fn aliased_parallel_step_uses_alias_line_for_inherited_child() {
    let findings = run("aliased-parallel-step", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 10);
    assert!(run("aliased-parallel-step-suppressed", "{}").is_empty());
}

#[test]
fn non_job_and_non_step_shapes_do_not_hide_authored_executable_steps() {
    assert!(run("no-jobs", "{}").is_empty());
    let findings = run("mixed-job-shapes", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 13);
    assert_eq!(
        findings[0].target.as_deref(),
        Some(".github/workflows/ci.yml#build#2")
    );
}

#[test]
fn no_jobs_still_audits_stale_and_duplicate_allows() {
    let findings = run(
        "no-jobs",
        r#"
allow:
  - step: ".github/workflows/ci.yml#removed#checkout"
    reason: "Former exception."
  - step: ".github/workflows/ci.yml#removed#checkout"
    reason: "Duplicate former exception."
"#,
    );
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("stale")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("duplicate")));
}

#[test]
fn removed_workflow_file_keeps_allow_audits_active() {
    let options = r#"
allow:
  - step: ".github/workflows/deleted.yml#build#checkout"
    reason: "Former exception."
  - step: ".github/workflows/deleted.yml#build#checkout"
    reason: "Duplicate former exception."
"#;
    let findings = run("deleted-workflow", options);
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("stale")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("duplicate")));
    assert_eq!(run_files("deleted-workflow", options, &[]).len(), 2);
}

#[test]
fn next_line_job_anchor_preserves_step_line_and_suppression() {
    let findings = run("next-line-job-anchor", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 7);
    assert!(run("next-line-job-anchor-suppressed", "{}").is_empty());
}

#[test]
fn anchor_properties_on_mapping_keys_preserve_nested_step_locations() {
    let findings = run("anchored-keys", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 7);
    assert!(run("anchored-keys-suppressed", "{}").is_empty());
}

#[test]
fn missing_workflow_source_is_skipped() {
    assert!(run_files("pass", "{}", &[".github/workflows/missing.yml"]).is_empty());
}

#[test]
fn named_flow_parallel_locates_declaration_after_comment() {
    let findings = run("named-flow-parallel", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 8);
    assert!(run("named-flow-parallel-suppressed", "{}").is_empty());
}

#[test]
fn flow_jobs_steps_and_parallel_share_declaration_line() {
    let findings = run("flow-all", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 2);
    assert!(run("flow-all-suppressed", "{}").is_empty());
}

#[test]
fn multiline_flow_sequences_preserve_each_executable_step_line() {
    for (scenario, line) in [("multiline-flow-steps", 7), ("multiline-flow-parallel", 9)] {
        let findings = run(scenario, "{}");
        assert_eq!(findings.len(), 1, "{scenario}: {findings:?}");
        assert_eq!(findings[0].line, line);
        assert!(run(&format!("{scenario}-suppressed"), "{}").is_empty());
    }
    let findings = run("multiline-flow-steps-key-suppressed", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 8);
}

#[test]
fn parallel_children_in_multiline_flow_mapping_keep_own_lines() {
    let findings = run("nested-flow-parallel", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 8);
    assert!(run("nested-flow-parallel-suppressed", "{}").is_empty());
}

#[test]
fn root_flow_mapping_locates_each_authored_step() {
    for (scenario, line) in [
        ("root-flow", 3),
        ("anchored-root-flow", 3),
        ("document-root-flow", 3),
        ("multiline-root-flow", 7),
    ] {
        let findings = run(scenario, "{}");
        assert_eq!(findings.len(), 1, "{scenario}: {findings:?}");
        assert_eq!(findings[0].line, line);
        assert!(run(&format!("{scenario}-suppressed"), "{}").is_empty());
    }
}

#[test]
fn block_job_with_multiline_flow_value_locates_child_steps() {
    let findings = run("multiline-flow-job", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 6);
    assert!(run("multiline-flow-job-suppressed", "{}").is_empty());
}

#[test]
fn next_line_multiline_flow_jobs_locates_child_steps() {
    let findings = run("multiline-next-line-flow-jobs", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 4);
    assert!(run("multiline-next-line-flow-jobs-suppressed", "{}").is_empty());
}

#[test]
fn anchored_multiline_flow_jobs_locates_child_steps() {
    let findings = run("same-line-multiline-flow-jobs", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 3);
    assert!(run("same-line-multiline-flow-jobs-suppressed", "{}").is_empty());
}

#[test]
fn root_flow_job_alias_uses_its_own_declaration() {
    let findings = run("root-flow-job-alias", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 6);
    assert_eq!(
        findings[0].target.as_deref(),
        Some(".github/workflows/ci.yml#build#1")
    );
    assert!(run("root-flow-job-alias-suppressed", "{}").is_empty());
}

#[test]
fn root_flow_jobs_alias_uses_its_declaration() {
    let findings = run("root-flow-jobs-alias", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 4);
    assert!(run("root-flow-jobs-alias-suppressed", "{}").is_empty());
}

#[test]
fn root_flow_next_line_jobs_alias_uses_its_declaration() {
    let findings = run("root-flow-next-line-jobs-alias", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 4);
    assert!(run("root-flow-next-line-jobs-alias-suppressed", "{}").is_empty());
}

#[test]
fn flow_parallel_alias_uses_group_line_for_inherited_child() {
    let findings = run("flow-parallel-alias", "{}");
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(findings.iter().map(|f| f.line).collect::<Vec<_>>(), [7, 8]);
    let suppressed = run("flow-parallel-alias-suppressed", "{}");
    assert_eq!(suppressed.len(), 1, "{suppressed:?}");
    assert_eq!(suppressed[0].line, 7);
}

#[test]
fn root_flow_next_line_steps_locate_each_child() {
    let findings = run("root-flow-next-line-steps", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 4);
    assert!(run("root-flow-next-line-steps-suppressed", "{}").is_empty());
}

#[test]
fn root_flow_next_line_job_alias_uses_job_declaration() {
    let findings = run("root-flow-next-line-job-alias", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 6);
    assert!(run("root-flow-next-line-job-alias-suppressed", "{}").is_empty());
}

#[test]
fn root_flow_step_alias_uses_alias_declaration() {
    let findings = run("root-flow-step-alias", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 6);
    assert!(run("root-flow-step-alias-suppressed", "{}").is_empty());
}

#[test]
fn indexed_jobs_keep_distinct_step_lines() {
    let findings = run("multi-jobs", "{}");
    assert_eq!(findings.len(), 3, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [6, 10, 14]
    );
}

#[test]
fn flow_mapping_comment_delimiter_does_not_hide_parallel_children() {
    let findings = run("nested-flow-comment", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 9);
    assert!(run("nested-flow-comment-suppressed", "{}").is_empty());
}

#[test]
fn next_line_flow_collections_use_declaration_lines() {
    for (scenario, line) in [
        ("next-line-flow-steps", 5),
        ("next-line-flow-parallel", 6),
        ("next-line-flow-jobs", 2),
    ] {
        let findings = run(scenario, "{}");
        assert_eq!(findings.len(), 1, "{scenario}: {findings:?}");
        assert_eq!(findings[0].line, line);
        assert!(run(&format!("{scenario}-suppressed"), "{}").is_empty());
    }
}

#[test]
fn next_line_anchored_flow_uses_declaration_line() {
    let findings = run("next-line-anchored-flow", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 5);
    assert!(run("next-line-anchored-flow-suppressed", "{}").is_empty());
}

#[test]
fn next_line_alias_uses_declaration_line() {
    let findings = run("next-line-alias", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 10);
    assert!(run("next-line-alias-suppressed", "{}").is_empty());
}

#[test]
fn indented_root_mapping_keeps_step_location() {
    let findings = run("indented-root", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 6);
    assert!(run("indented-root-suppressed", "{}").is_empty());
}

#[test]
fn matrix_steps_do_not_shift_executable_step_location() {
    let findings = run("matrix-steps", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 11);
    assert!(run("matrix-steps-suppressed", "{}").is_empty());
}
