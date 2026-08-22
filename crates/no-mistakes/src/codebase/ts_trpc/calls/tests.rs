use super::{extract_trpc_calls, TrpcCallFact};

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
