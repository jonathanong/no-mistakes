use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;

const CJS_FACTORIES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/created-require/cjs-factories.cjs"
));
const CJS_DYNAMIC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/created-require/cjs-dynamic.cjs"
));

fn cjs_facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::cjs()).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "parse errors: {:#?}",
        parsed.diagnostics
    );
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

fn offset(source: &str, token: &str) -> u32 {
    u32::try_from(source.find(token).expect("call in fixture")).expect("offset")
}

#[test]
fn cjs_create_require_factories_record_literal_loads_and_ignore_other_modules() {
    let facts = cjs_facts(CJS_FACTORIES);
    for (call, specifier) in [
        ("load(\"../widget.cjs\")", "../widget.cjs"),
        ("renamed.resolve(\"../item.cjs\")", "../item.cjs"),
        ("viaNs(\"../ns.cjs\")", "../ns.cjs"),
        ("viaDirect(\"../direct.cjs\")", "../direct.cjs"),
        ("viaProp(\"../prop.cjs\")", "../prop.cjs"),
        ("inner(\"../inner.cjs\")", "../inner.cjs"),
    ] {
        assert!(
            facts
                .namespaces
                .literal_import_spans
                .contains(&(offset(CJS_FACTORIES, call), specifier.to_string())),
            "missing {call}: {:?}",
            facts.namespaces.literal_import_spans
        );
    }
    assert!(
        facts
            .namespaces
            .literal_import_spans
            .iter()
            .all(|(_, spec)| spec != "../ignored.cjs"),
        "{:?}",
        facts.namespaces.literal_import_spans
    );
    assert_eq!(
        facts.namespaces.created_require_specifiers,
        [
            "../widget.cjs",
            "../ns.cjs",
            "../direct.cjs",
            "../prop.cjs",
            "../inner.cjs",
        ]
    );
    assert_eq!(
        facts.namespaces.literal_require_resolve_offsets,
        vec![offset(CJS_FACTORIES, "renamed.resolve(")]
    );
}

#[test]
fn cjs_dynamic_create_require_marks_computed_created_require() {
    let facts = cjs_facts(CJS_DYNAMIC);
    assert!(facts.namespaces.computed_created_require);
    assert!(facts.namespaces.created_require_specifiers.is_empty());
}
