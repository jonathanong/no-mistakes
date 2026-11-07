use super::*;
use crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands;
use oxc_allocator::Allocator;
use oxc_ast_visit::{walk, Visit};
use oxc_parser::Parser;
use oxc_span::SourceType;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/codebase-analysis/literal-sql-catalogs.ts"
));

#[test]
fn literal_catalog_callback_grammar_retains_paths_and_rejects_effects() {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, SOURCE, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let flow = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        SOURCE,
        false,
        false,
        true,
    )
    .value_flow;
    let summaries: FxHashMap<_, _> = parsed
        .program
        .body
        .iter()
        .filter_map(|statement| {
            let oxc_ast::ast::Statement::VariableDeclaration(declaration) = statement else {
                return None;
            };
            let binding = declaration.declarations.first()?;
            let BindingPattern::BindingIdentifier(name) = &binding.id else {
                return None;
            };
            let Expression::ArrowFunctionExpression(arrow) = binding.init.as_ref()? else {
                return None;
            };
            Some((name.name.as_str(), callbacks::callback(arrow)))
        })
        .collect();
    for (name, output, paths) in [
        ("unary", false, vec![vec!["relation"]]),
        (
            "conditional",
            true,
            vec![vec!["key"], vec!["relation", "table"]],
        ),
        (
            "addition",
            true,
            vec![vec!["key"], vec!["relation", "table"]],
        ),
        ("computed", true, vec![vec!["relation", "table"]]),
        ("numeric", false, vec![]),
        (
            "logical",
            false,
            vec![vec!["key"], vec!["relation", "table"]],
        ),
    ] {
        let binding = flow
            .bindings
            .iter()
            .find(|binding| binding.name == name)
            .unwrap();
        let ValueExpression::Callable(_) = binding.value else {
            panic!("{name}")
        };
        let callback = summaries
            .get(name)
            .unwrap()
            .as_ref()
            .unwrap_or_else(|| panic!("{name}"));
        let paths: Vec<Vec<String>> = paths
            .into_iter()
            .map(|path| path.into_iter().map(String::from).collect())
            .collect();
        assert_eq!(callback.reads, paths, "{name}");
        assert_eq!(callback.string_output, output, "{name}");
    }
    for name in [
        "unaryOther",
        "optionalMember",
        "block",
        "destructured",
        "asynchronous",
        "rest",
        "impure",
    ] {
        let binding = flow
            .bindings
            .iter()
            .find(|binding| binding.name == name)
            .unwrap();
        let ValueExpression::Callable(_) = binding.value else {
            panic!("{name}")
        };
        assert!(summaries.get(name).unwrap().is_none(), "{name}");
    }
    assert_eq!(flow.catalogs.sources.len(), 1);
    assert_eq!(flow.catalogs.sources[0].name, "CATALOG");
    assert_eq!(flow.catalogs.accessors.len(), 1);
    assert_eq!(flow.catalogs.accessors[0].source, "CATALOG");
}

#[derive(Default)]
struct DisabledHooks {
    collector: ImportCollector,
}

impl<'a> Visit<'a> for DisabledHooks {
    fn visit_variable_declarator(&mut self, declaration: &oxc_ast::ast::VariableDeclarator<'a>) {
        if let Some(init) = &declaration.init {
            self.collector.note_literal_catalog_binding(
                &declaration.id,
                init,
                declaration.span.start,
            );
        }
        walk::walk_variable_declarator(self, declaration);
    }

    fn visit_return_statement(&mut self, statement: &ReturnStatement<'a>) {
        self.collector.note_literal_catalog_return(statement);
        walk::walk_return_statement(self, statement);
    }

    fn visit_expression(&mut self, expression: &Expression<'a>) {
        self.collector.note_literal_catalog_expression(expression);
        walk::walk_expression(self, expression);
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.collector.note_literal_catalog_identifier(identifier);
        walk::walk_identifier_reference(self, identifier);
    }
}

#[test]
fn literal_catalog_hooks_honor_disabled_fact_demand() {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, SOURCE, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut visitor = DisabledHooks::default();
    visitor.visit_program(&parsed.program);
    assert_eq!(
        visitor.collector.value_flow.catalogs,
        LiteralCatalogFacts::default()
    );
}
