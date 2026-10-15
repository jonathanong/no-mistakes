use super::*;

#[test]
fn combined_history_preserves_each_dynamic_prefix_capture_and_coarse_only_writes() {
    let source = saved_history_source();
    let offset = source
        .find("box.outer.client.get = replacement();")
        .unwrap() as u32;
    let completed_at = offset + "box.outer.client.get = replacement()".len() as u32;
    let key = (0, "box".to_string());
    let path = [Some("outer".into()), None, Some("client".into()), None];
    let reads = vec![Some((offset - 1, None)); path.len()];
    let metadata = mutation_metadata::MutationMetadata {
        offset,
        context: None,
        fresh: true,
        unconditional: true,
        region: Some(17),
        epoch: 3,
        phase: Some((1, 2)),
        foreign_before: 0,
    };
    let mut collector = ImportCollector::default();
    collector
        .prior_member_mutations
        .require_reference_proof(key.0, &key.1);
    collector.remember_member_mutation(
        &key,
        &path,
        &reads,
        Some((Some("get".into()), completed_at)),
        &metadata,
        true,
    );
    let history = collector.prior_member_mutations.history(&key).unwrap().0;
    assert!(history.has_assignments);
    let outer = &history.members["outer"];
    let client = &outer.dynamic.as_ref().unwrap().members["client"];
    for prefix in [outer, client] {
        let unknown = &prefix
            .assigned
            .as_ref()
            .unwrap()
            .replacements
            .as_ref()
            .unwrap()
            .identity
            .as_ref()
            .unwrap()
            .unknown;
        assert!(unknown.changed(Some((offset, None)), Some((completed_at, None))));
        assert!(!unknown.changed(Some((offset, None)), Some((completed_at - 1, None))));
    }
    let leaf = client.dynamic.as_ref().unwrap();
    let assigned = leaf.assigned.as_ref().unwrap();
    let write = assigned.propagated.as_ref().unwrap().aggregate().members["get"];
    assert_eq!(write.offset, completed_at);
    assert_eq!(write.unconditional_offset.unwrap().get(), completed_at);
    let replacements = assigned.replacements.as_ref().unwrap();
    assert!(replacements.members["get"][&completed_at]);
    assert_eq!(
        replacements.regional_offset(17, "get", completed_at),
        Some(completed_at)
    );
    let capture = replacements.identity.as_ref().unwrap().captures[&completed_at]
        .as_ref()
        .unwrap();
    assert_eq!(capture.origin, Some((offset, None)));
    assert_eq!(capture.completed_at, completed_at);
    assert_eq!(capture.epoch, 3);
    assert_eq!(capture.phase, Some((1, 2)));
    assert_eq!(capture.reads.iter().copied().collect::<Vec<_>>(), reads);
    for prefix in [
        history,
        outer,
        outer.dynamic.as_ref().unwrap(),
        client,
        leaf,
    ] {
        assert_eq!(prefix.summary.as_ref().unwrap().offset, offset);
        assert_eq!(
            prefix.summary.as_ref().unwrap().contexts,
            [None].into_iter().collect()
        );
    }
    let mut coarse = ImportCollector::default();
    coarse.remember_member_mutation(&key, &path, &reads, None, &metadata, false);
    let history = coarse.prior_member_mutations.history(&key).unwrap().0;
    assert!(history.has_unassigned);
    assert!(!history.has_assignments);
    let leaf = history.members["outer"].dynamic.as_ref().unwrap().members["client"]
        .dynamic
        .as_ref()
        .unwrap();
    assert!(leaf.assigned.is_none());
    assert_eq!(leaf.summary.as_ref().unwrap().offset, offset);
}
