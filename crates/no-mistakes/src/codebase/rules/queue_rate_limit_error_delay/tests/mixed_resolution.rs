use super::super::matching::{target_fit, TargetFit};
use super::instance_target;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::dependencies::graph::{
    KnownCallTarget, ResolvedCallAlternatives, ResolvedCallCandidate, ResolvedCallSite,
    ResolvedCallTarget,
};
use crate::codebase::rules::no_single_enqueue_in_loop::modules::ResolvedModules;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

fn export(file: &str) -> KnownCallTarget {
    KnownCallTarget::ModuleExport {
        specifier: "./resolved".into(),
        export_path: "Worker.RateLimitError".into(),
        repository_target: Some((PathBuf::from(file), "Worker.RateLimitError".into())),
        callable_id: None,
    }
}

fn other() -> KnownCallTarget {
    KnownCallTarget::ModuleExport {
        specifier: "./other".into(),
        export_path: "Other".into(),
        repository_target: Some((PathBuf::from("/repo/src/other.ts"), "Other".into())),
        callable_id: None,
    }
}

fn site(targets: Vec<KnownCallTarget>) -> ResolvedCallSite {
    let known = targets
        .into_iter()
        .map(|target| ResolvedCallCandidate { target, node: None })
        .collect();
    ResolvedCallSite {
        file: PathBuf::from("/repo/src/caller.ts"),
        caller: None,
        caller_id: None,
        line: 4,
        offset: 0,
        invocation: InvocationKind::Construct,
        source_callee: "RateLimitError".into(),
        target: ResolvedCallTarget::Alternatives(Arc::new(ResolvedCallAlternatives::new(
            known, false,
        ))),
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    }
}

#[test]
fn mixed_candidates_are_neither_a_full_match_nor_a_miss() {
    let graph = from_raw_maps(PathBuf::from("/repo"), HashMap::new(), HashMap::new());
    let resolved = ResolvedModules::for_test(
        "glide-mq",
        BTreeSet::from([
            PathBuf::from("/repo/src/glide-mq.ts"),
            PathBuf::from("/repo/src/also-glide.ts"),
        ]),
    );
    let target = instance_target("delayMs");
    let fit = |targets| target_fit(&graph, &target, &site(targets), &resolved);
    assert_eq!(
        fit(vec![
            export("/repo/src/glide-mq.ts"),
            export("/repo/src/also-glide.ts"),
        ]),
        TargetFit::All
    );
    assert_eq!(fit(vec![other()]), TargetFit::None);
    assert_eq!(
        fit(vec![export("/repo/src/glide-mq.ts"), other()]),
        TargetFit::Mixed
    );
}
