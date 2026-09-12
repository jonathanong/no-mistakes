use super::*;
use no_mistakes::data_pw_query::DataPwHit;
use std::io::{self, Write};

fn report() -> DataPwReport {
    DataPwReport {
        value: "search-bar".to_string(),
        attributes: vec!["data-pw".to_string()],
        source: Some(vec![DataPwHit {
            file: "app/search.tsx".to_string(),
            line: 7,
            attribute: "data-pw".to_string(),
        }]),
        test: None,
    }
}

#[test]
fn deadline_after_iteration_discards_buffered_output() {
    let mut checks = 0;

    let error = render_report_with_deadline_check(&report(), Format::Paths, || {
        checks += 1;
        if checks == 2 {
            anyhow::bail!("synthetic timeout");
        }
        Ok(())
    })
    .unwrap_err();

    assert!(error.to_string().contains("synthetic timeout"));
    assert_eq!(checks, 2);
}

#[test]
fn buffered_human_and_markdown_outputs_preserve_sections() {
    let human = String::from_utf8(render_report(&report(), Format::Human).unwrap()).unwrap();
    let markdown = String::from_utf8(render_report(&report(), Format::Md).unwrap()).unwrap();

    assert_eq!(
        human,
        "search-bar (attributes: data-pw)\n  source (1)\n    app/search.tsx:7 [data-pw]\n"
    );
    assert_eq!(
        markdown,
        "# data-pw `search-bar`\n\n## Source\n- `app/search.tsx:7` (data-pw)\n"
    );
}

#[test]
fn buffered_structured_and_paths_outputs_preserve_formats() {
    let json = String::from_utf8(render_report(&report(), Format::Json).unwrap()).unwrap();
    let yaml = String::from_utf8(render_report(&report(), Format::Yml).unwrap()).unwrap();
    let paths = String::from_utf8(render_report(&report(), Format::Paths).unwrap()).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["value"], "search-bar");
    assert!(json.starts_with('{'));
    assert!(json.ends_with('\n'));
    assert_eq!(json.matches('\n').count(), 1);
    assert!(yaml.contains("value: search-bar"));
    assert!(yaml.ends_with("\n\n"));
    assert_eq!(paths, "app/search.tsx\n");
}

#[test]
fn data_pw_text_writers_surface_io_errors() {
    struct FailAfter {
        remaining_writes: usize,
    }
    impl Write for FailAfter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
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
    fn exhaust(mut write: impl FnMut(&mut FailAfter) -> io::Result<()>) {
        let mut completed = false;
        for remaining_writes in 0..64 {
            let mut writer = FailAfter { remaining_writes };
            if write(&mut writer).is_ok() {
                assert!(remaining_writes > 0);
                completed = true;
                break;
            }
        }
        assert!(completed, "writer should succeed after enough writes");
    }
    let populated = DataPwReport {
        value: "search-bar".to_string(),
        attributes: vec!["data-pw".to_string()],
        source: Some(vec![DataPwHit {
            file: "app/search.tsx".to_string(),
            line: 7,
            attribute: "data-pw".to_string(),
        }]),
        test: Some(vec![DataPwHit {
            file: "e2e/search.spec.ts".to_string(),
            line: 12,
            attribute: "data-pw".to_string(),
        }]),
    };
    let empty = DataPwReport {
        value: "missing".to_string(),
        attributes: vec!["data-pw".to_string()],
        source: None,
        test: None,
    };
    for report in [&populated, &empty] {
        exhaust(|writer| write_human(report, writer));
        exhaust(|writer| write_md(report, writer));
    }
}
