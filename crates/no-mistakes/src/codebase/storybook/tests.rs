use super::extract_program_with_references;
use crate::imports::collect_identifier_references;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::HashSet;

fn extract_program(source: &str, program: &oxc_ast::ast::Program<'_>) -> super::StorybookFileFacts {
    let referenced = collect_identifier_references(program);
    let referenced: HashSet<&str> = referenced.iter().map(String::as_str).collect();
    extract_program_with_references(source, program, &referenced)
}

#[test]
fn extracts_used_and_side_effect_story_imports() {
    let source = r#"
	import "./setup.story";
	import type TypeOnlyDefault from "./TypeOnlyDefault";
	import UsedDefault from "./Default";
	import UnusedDefault from "./Unused";
	import { Used, Unused, type TypeOnlyNamed } from "./Named";
	import * as Namespace from "./Namespace";

export const Basic = () => <><UsedDefault /><Used /><Namespace.Card /></>;
"#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
    let facts = extract_program(source, &parsed.program);

    assert_eq!(facts.side_effect_imports.len(), 1);
    assert_eq!(facts.side_effect_imports[0].source, "./setup.story");
    assert_eq!(
        facts
            .used_runtime_imports
            .iter()
            .map(|import| (import.imported.as_str(), import.local.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("default", "UsedDefault"),
            ("Used", "Used"),
            ("*", "Namespace"),
        ]
    );
}
