use super::extract_program;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn extracts_used_and_side_effect_story_imports() {
    let source = r#"
import "./setup.story";
import UsedDefault from "./Default";
import UnusedDefault from "./Unused";
import { Used, Unused } from "./Named";
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
