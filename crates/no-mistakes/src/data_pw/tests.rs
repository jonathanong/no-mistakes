use super::*;
use no_mistakes::data_pw_query::DataPwHit;

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
