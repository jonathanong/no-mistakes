use super::{finish_trpc_calls, procedure_path_from_call, TrpcCallFact};
use oxc_allocator::Allocator;
use oxc_ast::ast::CallExpression;
use oxc_ast_visit::{walk, Visit};
use oxc_span::SourceType;

fn extract_trpc_calls(source: &str) -> Vec<TrpcCallFact> {
    let allocator = Allocator::default();
    let ret = crate::ast::parse(
        std::path::Path::new("trpc-calls.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    let mut visitor = CallVisitor { calls: Vec::new() };
    visitor.visit_program(&ret.program);
    finish_trpc_calls(&mut visitor.calls);
    visitor.calls
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
