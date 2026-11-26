use super::*;

#[test]
fn capture_timeline_requires_all_root_write_completions() {
    let key = (1, "box".to_string());
    let mut collector = ImportCollector::default();
    assert!(collector.parameter_capture_writes_covered(&key, &fx_set()));
    let epochs = Epochs {
        writes: [(10, Some(CallableId(1))), (20, None)]
            .into_iter()
            .collect(),
        foreign: false,
    };
    collector
        .receiver_escapes
        .parameter_epochs
        .entry(key.0)
        .or_default()
        .insert(key.1.clone(), epochs);
    assert!(!collector.parameter_capture_writes_covered(&key, &[10].into_iter().collect()));
    assert!(collector.parameter_capture_writes_covered(&key, &[10, 20].into_iter().collect()));
    collector
        .receiver_escapes
        .parameter_epochs
        .get_mut(&key.0)
        .unwrap()
        .get_mut(&key.1)
        .unwrap()
        .foreign = true;
    assert!(!collector.parameter_capture_writes_covered(&key, &[10, 20].into_iter().collect()));
}
