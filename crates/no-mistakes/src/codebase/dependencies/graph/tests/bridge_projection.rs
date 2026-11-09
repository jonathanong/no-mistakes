use super::*;
use crate::codebase::dependencies::graph::{test_support, NodeId};
use std::collections::HashSet;

#[test]
fn excluded_file_projection_prunes_forward_and_reverse_bridges() {
    let files = ["/root/app.ts", "/root/qa.runner.ts", "/root/target.ts"];
    let forward = raw_fwd(&[
        (files[0], &[files[1]]),
        (files[1], &[files[2]]),
        (files[2], &[]),
    ]);
    let graph = test_support::from_raw_maps(p("/root"), forward, raw_rev(&[]));
    let excluded: HashSet<_> = [p(files[1])].into_iter().collect();
    let start = NodeId::file(p(files[0]));
    let target = NodeId::file(p(files[2]));

    assert_eq!(
        graph
            .deps_of(std::slice::from_ref(&start), None, None)
            .len(),
        2
    );
    assert!(graph
        .deps_of_excluding_files(&[start], None, &excluded)
        .is_empty());
    assert!(graph
        .dependents_of_excluding_files(&[target], None, None, &excluded)
        .is_empty());
}
