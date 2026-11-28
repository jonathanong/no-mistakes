//! Streaming, lossy LCOV parser with record-scoped diagnostics.

mod functions;
mod records;

use crate::merge::merge_file_coverage;
use crate::model::{CoverageReport, ParseDiagnostic, ParseDiagnosticKind, ParsedLcov};
use crate::normalize::{normalize_path, NormalizeOptions};
use records::{apply_data_record, classify_data_record, ActiveRecord};

/// Options controlling LCOV parsing and path normalization.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParseOptions {
    pub normalize: NormalizeOptions,
}

impl ParseOptions {
    pub fn with_normalize(normalize: NormalizeOptions) -> Self {
        Self { normalize }
    }
}

/// Parses raw LCOV bytes, decoding invalid UTF-8 lossily and keeping errors scoped to their record.
pub fn parse_lcov_bytes(bytes: &[u8], options: &ParseOptions) -> ParsedLcov {
    match std::str::from_utf8(bytes) {
        Ok(text) => parse_lcov_str(text, options),
        Err(err) => {
            let line = byte_offset_line(bytes, err.valid_up_to());
            let decoded = String::from_utf8_lossy(bytes);
            let mut parsed = parse_lcov_str(&decoded, options);
            parsed.diagnostics.insert(
                0,
                ParseDiagnostic {
                    line,
                    record_path: None,
                    kind: ParseDiagnosticKind::InvalidUtf8,
                    message: format!(
                        "invalid UTF-8 sequence decoded lossily at byte {}",
                        err.valid_up_to()
                    ),
                },
            );
            parsed
        }
    }
}

/// Parses UTF-8 LCOV text into a normalized report plus scoped parse diagnostics.
pub fn parse_lcov_str(text: &str, options: &ParseOptions) -> ParsedLcov {
    let mut report = CoverageReport::new();
    let mut diagnostics = Vec::new();
    let mut active: Option<ActiveRecord> = None;

    for (idx, raw_line) in text.split('\n').enumerate() {
        let line_no = idx + 1;
        let line = trim_line_end(raw_line);
        if line.is_empty() {
            continue;
        }

        if let Some(raw_path) = line.strip_prefix("SF:") {
            commit_record(active.take(), &mut report);
            match normalize_path(raw_path, &options.normalize) {
                Ok(path) => active = Some(ActiveRecord::new(path)),
                Err(err) => {
                    diagnostics.push(ParseDiagnostic {
                        line: line_no,
                        record_path: (!raw_path.is_empty()).then(|| raw_path.to_string()),
                        kind: ParseDiagnosticKind::InvalidPath,
                        message: err.to_string(),
                    });
                    active = Some(ActiveRecord::skipped());
                }
            }
            continue;
        }

        if line == "end_of_record" {
            commit_record(active.take(), &mut report);
            continue;
        }

        let Some(data_record) = classify_data_record(line) else {
            continue;
        };

        let Some(record) = active.as_mut() else {
            diagnostics.push(ParseDiagnostic {
                line: line_no,
                record_path: None,
                kind: ParseDiagnosticKind::OrphanRecord,
                message: format!("record outside SF..end_of_record: {line}"),
            });
            continue;
        };

        if record.path.is_none() || record.empty_placeholder {
            continue;
        }

        if let Err(reason) = apply_data_record(data_record, record) {
            diagnostics.push(ParseDiagnostic {
                line: line_no,
                record_path: record.path.clone(),
                kind: ParseDiagnosticKind::MalformedLine,
                message: format!("{reason}: {line}"),
            });
        }
    }

    commit_record(active, &mut report);

    ParsedLcov {
        report,
        diagnostics,
    }
}

fn commit_record(active: Option<ActiveRecord>, report: &mut CoverageReport) {
    let Some(record) = active else {
        return;
    };
    if record.empty_placeholder {
        return;
    }
    let Some(path) = record.path else {
        return;
    };
    let dest = report.files.entry(path).or_default();
    merge_file_coverage(dest, &record.coverage);
}

fn trim_line_end(line: &str) -> &str {
    line.trim_end_matches([' ', '\t', '\r'])
}

fn byte_offset_line(bytes: &[u8], offset: usize) -> usize {
    bytes[..offset].iter().filter(|&&b| b == b'\n').count() + 1
}
