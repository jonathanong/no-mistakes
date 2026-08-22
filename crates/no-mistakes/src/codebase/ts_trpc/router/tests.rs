use super::{extract_trpc_router, TrpcRouterFacts};

#[test]
fn nested_static_router_keys_become_procedure_paths() {
    let facts = extract_trpc_router(
        r#"
export const appRouter = router({
  user: {
    get: procedure.query(() => null),
    create: procedure.input(z.string()).mutation(() => null),
  },
});
"#,
    );
    assert_eq!(
        facts,
        TrpcRouterFacts {
            procedures: vec!["user.create".into(), "user.get".into()],
        }
    );
}

#[test]
fn create_trpc_router_aliases_are_recognized() {
    let facts = extract_trpc_router(
        r#"
export const appRouter = createTRPCRouter({
  health: publicProcedure.query(() => "ok"),
});
createTrpcRouter({ ready: procedure.query(() => null) });
t.router({ ping: procedure.query(() => null) });
"#,
    );
    assert_eq!(facts.procedures, vec!["health", "ping", "ready"]);
}

#[test]
fn computed_keys_and_non_procedure_leaves_are_skipped() {
    let facts = extract_trpc_router(
        r#"
const key = "user";
const extra = {};
router({
  [key]: { get: procedure.query(() => null) },
  nested: { helper: () => null, leaf: query() },
  ...extra,
});
notARouter({ skip: procedure.query(() => null) });
"#,
    );
    assert!(facts.procedures.is_empty());
}
