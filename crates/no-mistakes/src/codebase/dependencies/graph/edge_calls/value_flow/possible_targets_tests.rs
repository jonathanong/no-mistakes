use super::*;

fn site(target: ResolvedCallTarget) -> ResolvedCallSite {
    ResolvedCallSite {
        file: "saved-call.mts".into(),
        caller: None,
        caller_id: None,
        line: 1,
        offset: 0,
        invocation: InvocationKind::Call,
        source_callee: "call".into(),
        target,
        alternatives: vec![],
        returned_from: vec![("provider".into(), "make".into())],
        typed_from: vec![],
        returned_member_from: vec![],
        target_node: None,
    }
}

#[test]
fn widening_retains_each_target_kind_and_legacy_metadata() {
    for target in [
        ResolvedCallTarget::Global {
            name: "fetch".into(),
        },
        ResolvedCallTarget::ModuleExport {
            specifier: "provider".into(),
            export_path: "get".into(),
            repository_target: None,
            callable_id: None,
        },
        ResolvedCallTarget::RepositoryFunction {
            file: "repo.mts".into(),
            scope: "handler".into(),
        },
        ResolvedCallTarget::Alternatives(std::sync::Arc::new(ResolvedCallAlternatives::new(
            vec![ResolvedCallCandidate {
                target: KnownCallTarget::Global {
                    name: "fetch".into(),
                },
                node: None,
            }],
            false,
        ))),
    ] {
        let mut call = site(target);
        let expected = call
            .targets()
            .map(|candidate| format!("{:?}", candidate.target))
            .collect::<Vec<_>>();
        call.alternatives = vec![site(ResolvedCallTarget::Unknown)];
        let metadata = call.clone();
        widen(&mut call);
        assert!(call.has_unknown_target());
        assert_eq!(
            call.targets()
                .map(|candidate| format!("{:?}", candidate.target))
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(call.alternatives, metadata.alternatives);
        assert_eq!(call.returned_from, metadata.returned_from);
    }
    let mut unknown = site(ResolvedCallTarget::Unknown);
    widen(&mut unknown);
    assert_eq!(unknown.target, ResolvedCallTarget::Unknown);
}
