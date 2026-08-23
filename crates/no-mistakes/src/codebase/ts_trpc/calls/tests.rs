use super::{finish_trpc_calls, procedure_path_from_call, TrpcCallFact};
use crate::codebase::ts_source::facts::{domain, TsFactContext, TsFactPlan};
use oxc_allocator::Allocator;
use oxc_ast::ast::{CallExpression, Program};
use oxc_ast_visit::{walk, Visit};
use oxc_span::SourceType;
use std::path::Path;

fn extract_trpc_calls_from_program(program: &Program<'_>) -> Vec<TrpcCallFact> {
    let mut visitor = CallVisitor { calls: Vec::new() };
    visitor.visit_program(program);
    finish_trpc_calls(&mut visitor.calls);
    visitor.calls
}

fn extract_trpc_calls(source: &str) -> Vec<TrpcCallFact> {
    let allocator = Allocator::default();
    let ret = crate::ast::parse(
        Path::new("trpc-calls.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    extract_trpc_calls_from_program(&ret.program)
}

fn fused_trpc_calls(source: &str) -> Vec<TrpcCallFact> {
    let allocator = Allocator::default();
    let path = Path::new("trpc-calls.ts");
    let ret = crate::ast::parse(path, &allocator, source, SourceType::ts());
    domain::collect_domain_facts(
        &ret.program,
        path,
        source,
        TsFactPlan {
            trpc_calls: true,
            ..TsFactPlan::default()
        },
        &TsFactContext::default(),
    )
    .trpc_calls
}

struct CallVisitor {
    calls: Vec<TrpcCallFact>,
}

impl<'a> Visit<'a> for CallVisitor {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(path) = procedure_path_from_call(call) {
            self.calls.push(TrpcCallFact { path });
        }
        walk::walk_call_expression(self, call);
    }
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

#[test]
fn fused_walk_matches_standalone_trpc_call_facts() {
    // Fusion must keep complete TrpcCallFact output identical to the
    // historical standalone walker, including skipped computed/shallow calls.
    const CAPTURE: &str = r#"
await trpc.user.get.query();
await trpc.user.create.mutate({});
await client.user.get.mutation();
"#;
    const SKIP: &str = r#"
const ns = "user";
trpc[ns].get.query();
trpc.user.query();
other.query();
"#;
    for source in [CAPTURE, SKIP] {
        assert_eq!(extract_trpc_calls(source), fused_trpc_calls(source));
    }

    let fixture = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/trpc-basic/fixture/src/client.ts"),
    )
    .expect("tRPC client fixture");
    assert_eq!(extract_trpc_calls(&fixture), fused_trpc_calls(&fixture));
}
