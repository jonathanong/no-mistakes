//! Deterministic LCOV serialization (full and minimal forms).

use crate::model::CoverageReport;
use std::fmt::Write;

/// Serializes a coverage report to deterministic full-fidelity LCOV with recomputed summaries.
pub fn to_lcov(report: &CoverageReport) -> String {
    if report.files.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for (file, cov) in &report.files {
        let _ = writeln!(out, "TN:");
        let _ = writeln!(out, "SF:{file}");

        let mut fn_defs: Vec<(u32, &str)> = cov
            .functions
            .iter()
            .filter_map(|(name, data)| data.line.map(|line| (line, name.as_str())))
            .collect();
        fn_defs.sort_unstable();
        for (line, name) in fn_defs {
            let _ = writeln!(out, "FN:{line},{name}");
        }

        for (name, data) in &cov.functions {
            let _ = writeln!(out, "FNDA:{},{name}", data.hits);
        }

        let _ = writeln!(out, "FNF:{}", cov.functions_found());
        let _ = writeln!(out, "FNH:{}", cov.functions_hit());

        for (key, hits) in &cov.branches {
            let _ = writeln!(out, "BRDA:{},{hits}", key.as_lcov_key());
        }

        let _ = writeln!(out, "BRF:{}", cov.branches_found());
        let _ = writeln!(out, "BRH:{}", cov.branches_hit());

        for (line, hits) in &cov.lines {
            let _ = writeln!(out, "DA:{line},{hits}");
        }

        let _ = writeln!(out, "LF:{}", cov.lines_found());
        let _ = writeln!(out, "LH:{}", cov.lines_hit());
        let _ = writeln!(out, "end_of_record");
    }

    out
}

/// Serializes a coverage report to minimal `SF:` / `DA:` / `end_of_record` LCOV text.
///
/// Empty `SF:` records are preserved so patch projections distinguish files present
/// with zero changed lines from files absent from coverage.
pub fn to_lcov_minimal(report: &CoverageReport) -> String {
    if report.files.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for (file, cov) in &report.files {
        let _ = writeln!(out, "SF:{file}");
        for (line, hits) in &cov.lines {
            let _ = writeln!(out, "DA:{line},{hits}");
        }
        let _ = writeln!(out, "end_of_record");
    }

    out
}
