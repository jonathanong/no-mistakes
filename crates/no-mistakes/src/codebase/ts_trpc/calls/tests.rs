use super::{extract_trpc_calls_from_program, TrpcCallFact};
use oxc_allocator::Allocator;
use oxc_span::SourceType;

fn extract_trpc_calls(source: &str) -> Vec<TrpcCallFact> {
    let allocator = Allocator::default();
    let ret = crate::ast::parse(
        std::path::Path::new("trpc-calls.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    extract_trpc_calls_from_program(&ret.program)
}

#[test]
fn static_client_calls_capture_procedure_paths() {
    let calls = extract_trpc_calls(
        r#"
await trpc.user.get.query();
await trpc.user.create.mutate({});
await client.user.get.mutation();
"#,
    );
    assert_eq!(
        calls,
        vec![
            TrpcCallFact {
                path: "user.create".into()
            },
            TrpcCallFact {
                path: "user.get".into()
            },
        ]
    );
}

#[test]
fn computed_members_and_shallow_calls_are_skipped() {
    const SRC: &str = r#"
const ns = "user";
trpc[ns].get.query();
trpc.user.query();
other.query();
"#;
    assert!(extract_trpc_calls(SRC).is_empty());
}
