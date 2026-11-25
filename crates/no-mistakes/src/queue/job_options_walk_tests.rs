use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn with_resolver(test: impl FnOnce(SpreadResolver<'_, '_>, u32)) {
    let source =
        include_str!("../../../../test-cases/queue-ast-hop/job-option-memo/fixture/objects.ts");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let index = crate::queue::extract_factory::queue_index(&parsed.program, &Default::default());
    let resolver = SpreadResolver {
        bindings: &index.object_bindings,
        objects: FxHashMap::default(),
        members: FxHashMap::default(),
        traversal: DeclarationTraversal::default(),
    };
    test(resolver, parsed.program.span.end - 1);
}

#[test]
fn cyclic_option_keys_are_resolved_from_each_seed() {
    with_resolver(|mut resolver, at| {
        for name in ["first", "second"] {
            let evidence = resolver.option_keys(&JobOptionSpread {
                name: name.into(),
                at,
            });
            assert!(evidence.open);
            assert_eq!(
                evidence.keys.into_iter().collect::<Vec<_>>(),
                ["attempts", "priority"]
            );
        }
        assert!(resolver.objects.is_empty());
    });
}

#[test]
fn cyclic_template_members_stay_uncached_and_absence_is_cached() {
    with_resolver(|mut resolver, at| {
        let absent = JobOptionSpread {
            name: "withoutOptions".into(),
            at,
        };
        let (declaration, _) = resolver.value(&absent).unwrap();
        assert!(resolver.opts_member(&absent).is_none());

        let cyclic = JobOptionSpread {
            name: "template".into(),
            at,
        };
        assert!(resolver.opts_member(&cyclic).unwrap().open);

        assert_eq!(resolver.members.len(), 1);
        assert!(matches!(resolver.members.get(&declaration), Some(None)));
    });
}
