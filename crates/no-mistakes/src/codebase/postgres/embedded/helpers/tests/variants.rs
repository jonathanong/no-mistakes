//! Holes inside complete alternatives, statement-level builder helpers, and IIFEs.

use super::*;
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::fx::FxHashMap;

fn pending_for(
    facts: &EmbeddedSqlFileFacts,
    index: usize,
) -> Option<&super::super::PendingImportedSql> {
    let span = facts.call_spans[index];
    facts
        .pending_imported
        .iter()
        .find(|pending| pending.span == span)
}

fn filled(facts: &EmbeddedSqlFileFacts, index: usize, hole_text: &str) -> EmbeddedSqlCall {
    let pending = pending_for(facts, index).expect("pending");
    let holes = pending
        .sites
        .iter()
        .map(|(site, _)| {
            let marker = Marker {
                line: 40,
                path: Some(PathBuf::from("/helper.ts")),
                origin: 1000,
            };
            (*site, vec![plain(hole_text.to_string(), marker)])
        })
        .collect::<FxHashMap<_, _>>();
    super::super::splice::splice(&pending.template, &holes).expect("spliced")
}

#[test]
fn a_hole_after_a_conditional_is_kept_for_every_alternative() {
    let facts = extract("variant-holes.ts");
    // Public form: a Dynamic call with no alternatives, as before.
    assert!(facts.calls[0].variants.is_empty());
    let pending = pending_for(&facts, 0).expect("pending");
    assert_eq!(pending.template.variants.len(), 2);
    assert!(pending
        .template
        .variants
        .iter()
        .all(|variant| sentinel::contains(&variant.sql_text)));
    let call = filled(&facts, 0, " AND h = 1");
    assert_eq!(call.kind, facts.calls[0].kind);
    let texts = call
        .variants
        .iter()
        .map(|variant| variant.sql_text.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        texts,
        [
            "SELECT 1 FROM t WHERE x = sql_placeholder_1 AND y = 2 AND h = 1 AND z = sql_placeholder_2",
            "SELECT 1 FROM t WHERE x = sql_placeholder_1 AND h = 1 AND z = sql_placeholder_2",
        ]
    );
    for variant in &call.variants {
        assert_eq!(variant.sql_source_offsets.len(), variant.sql_text.len());
        assert_eq!(variant.recovered_placeholder_positions.len(), 2);
        // The helper's bytes keep the helper's origin, the caller's keep theirs.
        let at = variant.sql_text.find(" AND h").expect("hole text");
        assert_eq!(variant.sql_source_offsets[at], 1000);
        assert!(variant
            .sql_source_positions
            .iter()
            .any(|position| position.source_path.as_deref() == Some(Path::new("/helper.ts"))));
    }
}

#[test]
fn a_statement_level_builder_helper_extends_the_builder() {
    let facts = extract("variant-holes.ts");
    let sites = |index| {
        pending_for(&facts, index).map(|pending| {
            pending
                .sites
                .iter()
                .map(|(_, site)| site.args.clone())
                .collect::<Vec<_>>()
        })
    };
    let shape = |index| {
        sites(index).map(|groups| {
            groups
                .into_iter()
                .map(|args| {
                    args.into_iter()
                        .map(|arg| match arg {
                            SiteArg::Literal(literal) => SiteArg::Literal(literal.text.into()),
                            other => other,
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
    };
    let builder_call = Some(vec![vec![
        SiteArg::Builder,
        SiteArg::Literal("tier".into()),
    ]]);
    assert_eq!(shape(1), builder_call);
    // Inside a branch only the alternatives hold the helper's text.
    assert_eq!(shape(2), builder_call);
    assert_eq!(
        facts.calls[2].sql_text.as_deref(),
        Some("SELECT 1 FROM t WHERE true")
    );
    let call = filled(&facts, 1, " AND f = 1");
    assert_eq!(call.variants.len(), 2);
    assert!(call
        .variants
        .iter()
        .all(|variant| variant.sql_text.ends_with(" AND f = 1")));
}

#[test]
fn an_executor_call_is_not_a_builder_helper() {
    let facts = extract("variant-holes.ts");
    assert!(pending_for(&facts, 3).is_none());
    assert_eq!(facts.calls[3].kind, EmbeddedSqlKind::ImmutableLocal);
}

#[test]
fn an_iife_hands_its_returned_builder_to_the_const() {
    let facts = extract("iife.ts");
    assert_eq!(facts.calls.len(), 2);
    for call in &facts.calls {
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic);
        assert_eq!(call.variants.len(), 2, "{call:#?}");
    }
}

#[test]
fn a_function_iife_recovers_the_returned_builder_and_rejects_unsafe_shapes() {
    let facts = extract("function-iife.ts");
    let branching = facts
        .calls
        .iter()
        .find(|call| call.line == 12)
        .expect("function IIFE executor");
    assert_eq!(branching.kind, EmbeddedSqlKind::Dynamic, "{branching:#?}");
    assert_eq!(branching.variants.len(), 2, "{branching:#?}");

    let nested = facts
        .calls
        .iter()
        .find(|call| call.line == 26)
        .expect("nested function IIFE executor");
    assert_eq!(nested.kind, EmbeddedSqlKind::Dynamic, "{nested:#?}");
    assert_eq!(
        nested.sql_text.as_deref(),
        Some("SELECT p.account_id FROM view_active_plans p")
    );

    for call in facts.calls.iter().filter(|call| call.line > 26) {
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic, "{call:#?}");
        assert!(
            call.variants.is_empty(),
            "unsafe function IIFE must stay opaque: {call:#?}"
        );
    }
}

#[test]
fn unpublishing_origins_requires_matching_lengths() {
    use super::super::unpublish::unpublish_origins;
    assert!(unpublish_origins("abc", &[], &[1, 2]).is_none());
    assert_eq!(
        unpublish_origins("abc", &[], &[1, 2, 3]),
        Some(vec![1, 2, 3])
    );
    let (published, positions) =
        publish_placeholders_with_positions(format!("a{INTERNAL_PLACEHOLDER}1b"));
    let origins = (0..published.len() as u32).collect::<Vec<_>>();
    let restored = unpublish_origins(&published, &positions, &origins).expect("origins");
    assert_eq!(restored.len(), format!("a{INTERNAL_PLACEHOLDER}1b").len());
    assert_eq!(
        (restored[0], restored[1], *restored.last().unwrap()),
        (0, 1, published.len() as u32 - 1)
    );
}
