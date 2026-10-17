use super::sentinel::{self, MARK};
use super::split::{compose, coordinate, origin_at, plain, split, Marker};
use super::unpublish::unpublish;
use super::{HelperSummary, ImportedSite, SiteArg};
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::postgres::embedded::{
    extract_embedded_sql_from_source, publish_placeholders_with_positions, EmbeddedSqlFileFacts,
    EmbeddedSqlKind, EmbeddedSqlOptions, INTERNAL_PLACEHOLDER,
};
use std::path::{Path, PathBuf};

#[path = "tests/pending.rs"]
mod pending;
#[path = "tests/summaries.rs"]
mod summaries;
#[path = "tests/variants.rs"]
mod variants;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/helpers")
        .join(name)
}

fn extract(name: &str) -> EmbeddedSqlFileFacts {
    let path = fixture(name);
    let source = std::fs::read_to_string(&path).expect("fixture");
    extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    )
}

fn caller(sql: String, line: u32) -> RecoveredSql {
    RecoveredSql {
        origins: vec![0; sql.len()],
        sql,
        line,
        positions: Vec::new(),
    }
}

#[test]
fn sentinel_markers_round_trip_and_ignore_malformed_text() {
    let text = format!("a{}b{}c", sentinel::encode(3), sentinel::encode(12));
    let sites = sentinel::find_all(&text)
        .into_iter()
        .map(|(_, site)| site)
        .collect::<Vec<_>>();
    assert_eq!(sites, [3, 12]);
    assert_eq!(sentinel::prefix(&text), "a");
    assert_eq!(sentinel::prefix("no hole"), "no hole");
    assert!(sentinel::contains(&text));
    for malformed in [
        format!("bare {MARK} marker"),
        format!("{MARK}12 never closed"),
        format!("{MARK}{MARK}"),
        format!("{MARK}99999999999999999999{MARK}"),
    ] {
        assert!(!sentinel::contains(&malformed), "{malformed:?}");
    }
}

#[test]
fn unpublish_restores_only_the_recorded_markers() {
    let internal =
        format!("a {INTERNAL_PLACEHOLDER}1 sql_placeholder_9 \n{INTERNAL_PLACEHOLDER}2 z");
    let (published, positions) = publish_placeholders_with_positions(internal.clone());
    // The user-authored spelling has no recorded position, so it stays as written.
    assert_eq!(
        unpublish(&published, &positions).as_deref(),
        Some(internal.as_str())
    );
    assert_eq!(unpublish("plain text", &[]).as_deref(), Some("plain text"));
    assert!(unpublish(&published, &[(1, 2)]).is_none());
    assert!(unpublish("short", &[(1, 9)]).is_none());
    assert!(unpublish("sql_placeholder_", &[(1, 1)]).is_none());
}

#[test]
fn split_and_compose_keep_every_physical_line() {
    let hole = sentinel::encode(0);
    let sql = format!("SELECT 1\nWHERE {hole} ORDER BY z");
    let mut joined = caller(sql, 10);
    // The second SQL line was authored far from the first.
    joined.positions = [(1, 10), (2, 50)]
        .map(
            |(sql_line, source_line)| crate::codebase::postgres::EmbeddedSqlSourcePosition {
                sql_line,
                sql_column: 1,
                source_line,
                source_path: None,
            },
        )
        .to_vec();
    let markers = sentinel::find_all(&joined.sql);
    let ranges = markers
        .iter()
        .map(|(range, _)| range.clone())
        .collect::<Vec<_>>();
    let (pieces, origins) = split(&joined, &ranges);
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0].sql, "SELECT 1\nWHERE ");
    assert_eq!(pieces[1].sql, " ORDER BY z");
    // The marker sits on the caller's second line.
    assert_eq!((origins[0].line, origins[0].path.clone()), (50, None));
    let helper = plain(
        "EXISTS (SELECT 1\nFROM t)".to_string(),
        Marker {
            line: 40,
            path: Some(PathBuf::from("/helper.ts")),
            origin: 7,
        },
    );
    let [first, second]: [RecoveredSql; 2] = pieces.try_into().ok().expect("two pieces");
    let whole = compose([first, helper, second]).expect("composed");
    assert_eq!(
        whole.sql,
        "SELECT 1\nWHERE EXISTS (SELECT 1\nFROM t) ORDER BY z"
    );
    let line_of = |needle: &str| {
        let offset = whole.sql.find(needle).expect(needle);
        origin_at(&whole.positions, whole.line, coordinate(&whole.sql, offset))
    };
    let helper_path = Some(PathBuf::from("/helper.ts"));
    assert_eq!(line_of("SELECT 1\nWHERE"), (10, None));
    assert_eq!(line_of("WHERE"), (50, None));
    assert_eq!(line_of("EXISTS"), (40, helper_path.clone()));
    assert_eq!(line_of("FROM t"), (41, helper_path));
    // Text after the helper returns to the caller's own lines and module.
    assert_eq!(line_of("ORDER"), (50, None));
}

#[test]
fn empty_pieces_and_empty_compositions_contribute_nothing() {
    assert!(compose([]).is_none());
    assert!(compose([caller(String::new(), 1)]).is_none());
    let only = compose([caller(String::new(), 1), caller("x".into(), 5)]).expect("one part");
    assert_eq!((only.sql.as_str(), only.line), ("x", 5));
    assert_eq!(only.positions.len(), 1);
}

#[test]
fn site_arguments_decide_which_summary_shapes_may_fill_a_hole() {
    let owner = Path::new("/helpers.ts");
    let site = |args: Vec<SiteArg>| ImportedSite {
        local: "helper".into(),
        args,
        line: 1,
    };
    let fixed = HelperSummary::Static(plain(
        "TRUE".into(),
        Marker {
            line: 3,
            path: None,
            origin: 0,
        },
    ));
    let literal = site(vec![SiteArg::Literal("a".into()), SiteArg::Scalar]);
    assert_eq!(fixed.fill(&literal, owner).expect("static").sql, "TRUE");
    // A builder argument means the helper's result includes the builder's text.
    assert!(fixed.fill(&site(vec![SiteArg::Builder]), owner).is_none());
    let filled = fixed.fill(&site(Vec::new()), owner).expect("no arguments");
    assert!(filled
        .positions
        .iter()
        .all(|position| position.source_path.as_deref() == Some(owner)));
}

fn template_call(internal: String) -> crate::codebase::postgres::EmbeddedSqlCall {
    let (published, positions) = publish_placeholders_with_positions(internal);
    crate::codebase::postgres::EmbeddedSqlCall {
        line: 9,
        kind: EmbeddedSqlKind::Composed,
        declaration_line: Some(9),
        sql_text: Some(published),
        recovered_placeholder_positions: positions,
        ..Default::default()
    }
}

#[test]
fn splicing_fills_every_hole_and_renumbers_the_generated_binds() {
    let p = INTERNAL_PLACEHOLDER;
    let internal = format!(
        "SELECT 1 WHERE a = {p}1 AND {} AND b = {p}2",
        sentinel::encode(0)
    );
    let template = template_call(internal);
    let owner = Some(PathBuf::from("/helper.ts"));
    let hole = plain(
        format!("c = {p}1"),
        Marker {
            line: 3,
            path: owner.clone(),
            origin: 0,
        },
    );
    let mut holes = crate::fx::FxHashMap::default();
    holes.insert(0, hole);
    let call = super::splice::splice(&template, &holes).expect("complete");
    assert_eq!(
        call.sql_text.as_deref(),
        Some(
            "SELECT 1 WHERE a = sql_placeholder_1 AND c = sql_placeholder_2 AND b = sql_placeholder_3"
        )
    );
    assert_eq!(call.kind, EmbeddedSqlKind::Composed);
    assert_eq!(call.recovered_placeholder_positions.len(), 3);
    assert!(call
        .sql_source_positions
        .iter()
        .any(|position| position.source_path == owner && position.source_line == 3));
    // Any gap leaves the call as the walk published it.
    assert!(super::splice::splice(&template, &Default::default()).is_none());
    let mut no_text = template.clone();
    no_text.sql_text = None;
    assert!(super::splice::splice(&no_text, &holes).is_none());
    let mut drifted = template.clone();
    drifted.recovered_placeholder_positions = vec![(1, 2)];
    assert!(super::splice::splice(&drifted, &holes).is_none());
    // A template holding no hole at all has nothing to compose from.
    let empty = template_call(String::new());
    assert!(super::splice::splice(&empty, &holes).is_none());
}

#[test]
fn extracted_calls_never_expose_a_hole_marker() {
    let facts = extract("pending.ts");
    let leaks = |text: &str| text.contains(MARK);
    assert!(facts
        .calls
        .iter()
        .all(|call| call.sql_text.as_deref().is_none_or(|text| !leaks(text))));
    assert!(facts
        .fragments
        .iter()
        .all(|fragment| fragment.sql_text.as_deref().is_none_or(|text| !leaks(text))));
    // Whatever still waits on a helper is, until it resolves, plainly `Dynamic`.
    assert!(facts
        .calls
        .iter()
        .zip(&facts.call_spans)
        .filter(|(_, span)| facts
            .pending_imported
            .iter()
            .any(|pending| pending.span == **span))
        .all(|(call, _)| call.kind == EmbeddedSqlKind::Dynamic));
}
