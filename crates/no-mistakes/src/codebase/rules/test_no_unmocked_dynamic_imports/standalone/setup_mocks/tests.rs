use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn each_distinct_ordered_context_is_traversed_once() {
    let calls = AtomicUsize::new(0);
    let first = vec![PathBuf::from("first.mts"), PathBuf::from("second.mts")];
    let reversed = first.iter().rev().cloned().collect::<Vec<_>>();
    // Repeated tests share a closure, but reversing setups changes execution.
    let contexts = [
        first.clone(),
        first.clone(),
        reversed.clone(),
        Vec::new(),
        Vec::new(),
    ];
    let cache = precompute_contexts(contexts.into_iter(), |files| {
        calls.fetch_add(1, Ordering::Relaxed);
        Ok(files.iter().cloned().collect())
    })
    .unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 3);
    assert!(cache.contains_key(&first));
    assert!(cache.contains_key(&reversed));
    assert!(cache[&Vec::new()].is_empty());
}

#[test]
fn setup_closure_failures_are_preserved() {
    let error = precompute_contexts(std::iter::once(vec![PathBuf::from("setup.mts")]), |_| {
        anyhow::bail!("fixture setup helper failure")
    })
    .unwrap_err();
    assert_eq!(error.to_string(), "fixture setup helper failure");
}
