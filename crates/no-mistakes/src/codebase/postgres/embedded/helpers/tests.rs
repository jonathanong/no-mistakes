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

#[path = "tests/finalize.rs"]
mod finalize;
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

fn splits_identifier(text: &str) -> bool {
    sentinel::find_all(text).iter().any(|(range, _)| {
        text[..range.start]
            .chars()
            .next_back()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    })
}

fn has_identifier_slot_hole(text: &str) -> bool {
    sentinel::find_all(text)
        .iter()
        .any(|(range, _)| sentinel::in_identifier_slot(text, range.start))
}

fn has_value_operand_hole(text: &str) -> bool {
    sentinel::find_all(text)
        .iter()
        .any(|(range, _)| text[..range.start].trim_end().ends_with('='))
}

fn keeps_placeholder(text: &str) -> bool {
    has_value_operand_hole(text) || (splits_identifier(text) && has_identifier_slot_hole(text))
}

fn restore_placeholders(text: &str) -> String {
    let holes = sentinel::find_all(text);
    if holes.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    for (index, (range, _)) in holes.iter().enumerate() {
        out.push_str(&text[cursor..range.start]);
        out.push_str("sql_placeholder_");
        out.push_str(&(index + 1).to_string());
        cursor = range.end;
    }
    out.push_str(&text[cursor..]);
    out
}

fn caller(sql: String, line: u32) -> RecoveredSql {
    RecoveredSql {
        invocations: Default::default(),
        origins: vec![0; sql.len()],
        sql,
        line,
        positions: Vec::new(),
    }
}

fn splice(
    template: &crate::codebase::postgres::EmbeddedSqlCall,
    holes: &crate::fx::FxHashMap<usize, Vec<RecoveredSql>>,
) -> Option<crate::codebase::postgres::EmbeddedSqlCall> {
    super::splice::splice_grouped(
        template,
        holes,
        &crate::fx::FxHashMap::default(),
        crate::codebase::postgres::MAX_EMBEDDED_SQL_VARIANTS,
    )
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

include!("tests_part1.rs");

include!("tests_part2.rs");
