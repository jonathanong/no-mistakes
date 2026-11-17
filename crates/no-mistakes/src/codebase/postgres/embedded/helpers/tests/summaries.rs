//! What the walk summarizes for importers of this file.

use super::*;
use crate::codebase::postgres::embedded::helpers::{BuilderHelper, LiteralArg};

fn summary(facts: &EmbeddedSqlFileFacts, name: &str) -> Option<HelperSummary> {
    facts.exported_helpers.get(name).cloned()
}

fn literals(values: &[&str]) -> ImportedSite {
    ImportedSite {
        local: "helper".into(),
        member: None,
        kind: crate::codebase::postgres::embedded::helpers::SiteKind::Sql,
        args: values
            .iter()
            .map(|value| SiteArg::Literal((*value).into()))
            .collect(),
        value_used: false,
        line: 1,
        constant: None,
    }
}

#[path = "summaries_cases/part1.rs"]
mod part1;

#[path = "summaries_cases/part2.rs"]
mod part2;
