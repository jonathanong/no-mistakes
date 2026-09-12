use super::*;
use crate::cli::Format;
use std::io::{self, Write};

struct FailAfter {
    remaining_writes: usize,
    attempted: bool,
}

impl Write for FailAfter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.attempted = true;
        if self.remaining_writes == 0 {
            return Err(io::Error::other("synthetic write failure"));
        }
        self.remaining_writes -= 1;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn exhaust(report: &SignatureImpactReport, format: Format) {
    let mut completed = false;
    for remaining_writes in 0..4096 {
        let mut writer = FailAfter {
            remaining_writes,
            attempted: false,
        };
        let result = write_report(report, format, &mut writer);
        if remaining_writes == 0 && writer.attempted {
            assert!(result.is_err(), "{format:?} zero-budget write must fail");
        }
        if result.is_ok() {
            completed = true;
            break;
        }
    }
    assert!(completed, "{format:?} should succeed after enough writes");
}

#[test]
fn impact_report_writers_surface_io_errors() {
    let populated = SignatureImpactReport {
        roots: vec!["src/date.mts".to_string()],
        symbol: "parseDate".to_string(),
        definition: SymbolLocation {
            file: "src/date.mts".to_string(),
            symbol: "parseDate".to_string(),
            line: 1,
            kind: "const",
        },
        exports: vec![SymbolLocation {
            file: "src/date.mts".to_string(),
            symbol: "parseDate".to_string(),
            line: 1,
            kind: "const",
        }],
        production_callers: vec![CallerEntry {
            file: "src/app.mts".to_string(),
            symbol: Some("run".to_string()),
            depth: 1,
            via: vec!["import"],
        }],
        test_callers: vec![CallerEntry {
            file: "src/date.test.mts".to_string(),
            symbol: None,
            depth: 1,
            via: vec!["test"],
        }],
        suggested_tests: vec![TestSuggestion {
            file: "src/date.test.mts".to_string(),
            depth: 1,
            via: vec!["test"],
        }],
        warnings: vec![],
    };
    let empty = SignatureImpactReport {
        roots: vec![],
        symbol: "parseDate".to_string(),
        definition: populated.definition.clone(),
        exports: vec![],
        production_callers: vec![],
        test_callers: vec![],
        suggested_tests: vec![],
        warnings: vec![],
    };
    for format in [
        Format::Md,
        Format::Human,
        Format::Paths,
        Format::Json,
        Format::Yml,
    ] {
        exhaust(&populated, format);
        exhaust(&empty, format);
    }
}
