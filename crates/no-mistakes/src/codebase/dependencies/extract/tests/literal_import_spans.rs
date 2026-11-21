use super::namespace_facts::namespaces;

#[test]
fn literal_import_spans_follow_the_import_keyword_in_every_position() {
    let source = concat!(
        "async function load() {\n",
        "  const { helper } = await import('./helper.ts')\n",
        "}\n",
        "vi.mock<typeof import('./dep.ts')>(import('./dep.ts'), async () => {\n",
        "  return import('./wrap.ts')\n",
        "})\n",
        "const parts = require('./columns.ts')\n",
        "type Dep = import('./type-dep.ts')\n",
    );
    let facts = namespaces(source);
    assert_eq!(facts.literal_import_spans, quoted_loads(source));
}

#[test]
fn a_non_literal_import_or_require_has_no_literal_span() {
    let source = concat!(
        "const name = './columns'\n",
        "const parts = import(`./${name}`)\n",
        "const other = require(name)\n",
    );
    assert!(namespaces(source).literal_import_spans.is_empty());
}

#[test]
fn a_star_namespace_reexport_records_its_own_offset() {
    let source = "export * as columns from './columns'\n";
    let facts = namespaces(source);
    let offset = u32::try_from(source.find("export").expect("export")).expect("offset");
    assert!(facts.export_name_offsets.is_empty());
    assert_eq!(
        facts.sourced_export_offsets,
        vec![(
            "./columns".to_string(),
            "*".to_string(),
            "columns".to_string(),
            offset
        )]
    );
}

fn quoted_loads(source: &str) -> Vec<(u32, String)> {
    let mut found = Vec::new();
    let mut rest = source;
    let mut base = 0usize;
    while let Some(index) = next_load(rest) {
        let token_len = if rest[index..].starts_with("import(") {
            "import(".len()
        } else {
            "require(".len()
        };
        let after = &rest[index + token_len..];
        let quote = after.chars().next().expect("quote");
        let body = &after[quote.len_utf8()..];
        let end = body.find(quote).expect("close quote");
        let at = u32::try_from(base + index).expect("offset");
        found.push((at, body[..end].to_string()));
        let advance = index + token_len;
        base += advance;
        rest = &rest[advance..];
    }
    found.sort();
    found
}

fn next_load(rest: &str) -> Option<usize> {
    match (rest.find("import("), rest.find("require(")) {
        (Some(import_at), Some(require_at)) => Some(import_at.min(require_at)),
        (Some(import_at), None) => Some(import_at),
        (None, Some(require_at)) => Some(require_at),
        (None, None) => None,
    }
}
