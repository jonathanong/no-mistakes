use super::*;
use crate::codebase::postgres::SiteKind;

fn at(
    calls: &[crate::codebase::postgres::EmbeddedSqlCall],
    line: u32,
) -> &crate::codebase::postgres::EmbeddedSqlCall {
    calls
        .iter()
        .find(|call| call.line == line)
        .expect("saved call")
}

#[test]
fn prepared_plain_clause_joins_only_complete_literal_switch_paths() {
    let before = prepared("prepared-plain-clause");
    let caller = before
        .iter()
        .find(|(path, _)| path.ends_with("src/caller.mts"))
        .unwrap()
        .1;
    let pending = &caller.embedded_sql[0].1.pending_imported;
    for line in [22, 25, 28] {
        assert!(
            pending.iter().any(|call| call.template.line == line
                && call
                    .sites
                    .iter()
                    .any(|(_, site)| site.kind == SiteKind::PlainClause)),
            "line {line}: {pending:#?}"
        );
    }
    for line in [54, 57, 61, 65, 96] {
        assert!(
            !pending.iter().any(|call| call.template.line == line
                && call
                    .sites
                    .iter()
                    .any(|(_, site)| site.kind == SiteKind::PlainClause)),
            "line {line}: {pending:#?}"
        );
    }

    let root = variant_root("prepared-plain-clause");
    crate::ast::begin_parse_count(&root);
    let mut files = prepared("prepared-plain-clause");
    let resolutions = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &resolutions);
    project(&mut files, resolve);
    let parses = crate::ast::finish_parse_count(&root);
    assert_eq!(parses.len(), 3, "{parses:#?}");
    assert!(parses.values().all(|count| *count == 1), "{parses:#?}");
    let calls = caller_calls(&files);
    for line in [22, 25, 28] {
        let call = at(&calls, line);
        assert_eq!(call.variants.len(), 7, "line {line}: {call:#?}");
        assert!(call
            .variants
            .iter()
            .any(|variant| variant.sql_text.contains("orders")));
        assert!(call
            .variants
            .iter()
            .all(|variant| !variant.sql_text.contains("\u{10FFFE}")));
    }
    for line in [39, 49, 54, 57, 61, 91, 96] {
        assert!(
            at(&calls, line).is_unanalyzable(),
            "line {line}: {calls:#?}"
        );
    }
    let tagged = at(&calls, 65);
    assert!(
        tagged
            .sql_text
            .as_deref()
            .is_none_or(|sql| !sql.contains("orders")),
        "{tagged:#?}"
    );
    assert!(
        tagged
            .variants
            .iter()
            .all(|variant| !variant.sql_text.contains("orders")),
        "{tagged:#?}"
    );
}

#[test]
fn plain_clause_arguments_admit_inert_templates_but_reject_spreads() {
    let files = run("prepared-clause-arguments");
    let calls = caller_calls(&files);
    let static_template = at(&calls, 13);
    assert!(
        static_template
            .sql_text
            .as_deref()
            .is_some_and(|text| text.contains("orders"))
            || static_template
                .variants
                .iter()
                .any(|variant| variant.sql_text.contains("orders"))
    );
    assert!(!static_template.is_unanalyzable());
    assert!(at(&calls, 18).is_unanalyzable());
}
