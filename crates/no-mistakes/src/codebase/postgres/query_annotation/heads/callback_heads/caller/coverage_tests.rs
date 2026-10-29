use super::{append_helpers, inline, local_payloads, Bindings, Span};
use crate::codebase::postgres::query_annotation::heads::syntax;
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};
use crate::fx::FxHashSet;
use oxc_ast::ast::{CallExpression, Expression};
use oxc_ast_visit::{walk, Visit};

fn saved(name: &str) -> (std::path::PathBuf, String) {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    (path, source)
}
#[test]
fn inline_census_and_preflight_keep_distinct_fail_closed_boundaries() {
    let (path, source) = saved("callback-inline-coverage.cjs");
    let spans = |name: &str| {
        source
            .match_indices(&format!("{name}("))
            .map(|(start, _)| (start as u32, (start + name.len()) as u32))
            .collect::<FxHashSet<_>>()
    };
    let helper = spans("appendHelper");
    struct Probes<'a> {
        source: &'a str,
        helpers: FxHashSet<Span>,
        count: usize,
    }
    impl<'a> Visit<'a> for Probes<'_> {
        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            if matches!(&call.callee, Expression::Identifier(id) if id.name == "probe") {
                let [full, cheap, value] = call.arguments.as_slice() else {
                    panic!("probe arity")
                };
                let Expression::BooleanLiteral(full) = full.as_expression().unwrap() else {
                    panic!("full oracle")
                };
                let Expression::BooleanLiteral(cheap) = cheap.as_expression().unwrap() else {
                    panic!("cheap oracle")
                };
                struct Endpoints(FxHashSet<Span>);
                impl<'a> Visit<'a> for Endpoints {
                    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
                        if matches!(&call.callee, Expression::Identifier(id) if id.name == "read") {
                            self.0.insert((call.span.start, call.span.end));
                        }
                        walk::walk_call_expression(self, call);
                    }
                }
                let value = value.as_expression().unwrap();
                let mut endpoints = Endpoints(Default::default());
                endpoints.visit_expression(value);
                let line = &self.source[call.span.start as usize..call.span.end as usize];
                assert_eq!(
                    inline::executor(value, &endpoints.0, &self.helpers).is_some(),
                    full.value,
                    "{line}"
                );
                assert_eq!(
                    inline::may_execute(value, &endpoints.0),
                    cheap.value,
                    "{line}"
                );
                self.count += 1;
            }
            walk::walk_call_expression(self, call);
        }
    }
    crate::ast::with_program(&path, &source, |program, _| {
        let mut probes = Probes {
            source: &source,
            helpers: helper,
            count: 0,
        };
        probes.visit_program(program);
        assert_eq!(probes.count, 27);
    })
    .unwrap();
}

#[test]
fn append_helper_and_local_payload_indexes_reject_nonclosed_owners() {
    let (path, source) = saved("callback-helper-coverage.mts");
    crate::ast::with_program(&path, &source, |program, _| {
        let facts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                imports: true,
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        let module = facts.module_bindings.as_deref().unwrap();
        let mut syntax = syntax::collect_with_helpers(program, true, true, true);
        let bindings = Bindings::new(&syntax, &facts, std::iter::empty(), true);
        let helpers = append_helpers::references(program, module, &syntax, &bindings);
        assert_eq!(helpers.len(), 1);
        let payloads = local_payloads::references(&syntax, &bindings, module, &helpers);
        assert_eq!(payloads.len(), 1);
        let start = source.find("consume(single)").unwrap() as u32 + 8;
        assert!(payloads.contains(&(start, start + 6)));
        let mut missing = module.clone();
        missing.bindings.retain(|binding| binding.name != "good");
        assert!(append_helpers::references(program, &missing, &syntax, &bindings).is_empty());
        let mut missing = module.clone();
        let parameter = missing
            .bindings
            .iter()
            .find(|binding| binding.name == "good")
            .unwrap()
            .span;
        missing.bindings.retain(|binding| {
            !(binding.name == "query"
                && binding.span.start > parameter.end
                && binding.span.start < parameter.end + 12)
        });
        assert!(append_helpers::references(program, &missing, &syntax, &bindings).is_empty());
        drop(bindings);
        syntax.dynamic_refs.push((0, 1));
        let bindings = Bindings::new(&syntax, &facts, std::iter::empty(), true);
        assert!(append_helpers::references(program, module, &syntax, &bindings).is_empty());
    })
    .unwrap();
}

#[test]
fn trusted_but_unnamed_callback_payload_has_no_fixed_annotation() {
    use super::super::uses::Uses;
    use oxc_ast::ast::{Declaration, Statement};
    let (path, source) = saved("callback-producer-coverage.mts");
    let tags = source
        .match_indices("sql`")
        .map(|(start, _)| (start as u32, start as u32 + 3))
        .collect();
    crate::ast::with_program(&path, &source, |program, _| {
        let Statement::ExportDeclaration(export) = program.body.last().unwrap() else {
            panic!("export")
        };
        let Declaration::FunctionDeclaration(function) = &export.declaration else {
            panic!("function")
        };
        assert!(Uses::inspect(
            function.body.as_ref().unwrap(),
            vec!["base", "execute"],
            1,
            &tags
        )
        .is_none());
    })
    .unwrap();
}
