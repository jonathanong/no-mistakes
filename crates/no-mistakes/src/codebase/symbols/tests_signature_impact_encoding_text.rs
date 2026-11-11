#[test]
fn signature_impact_text_reports_unanalyzed_files() {
    let fixture = encoding_fixture("tolerated");
    let root = fixture.path().canonicalize().unwrap();
    let report = impact::collect_report(&encoding_args(&root)).unwrap();
    for (format, name) in [
        (Format::Human, "signature_impact_encoding_human"),
        (Format::Md, "signature_impact_encoding_markdown"),
    ] {
        let mut output = Vec::new();
        impact::write_report(&report, format, &mut output).unwrap();
        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("unanalyzed-files"), "{rendered}");
        assert!(rendered.contains("binary.mts"), "{rendered}");
        assert!(rendered.contains("unrelated.mts"), "{rendered}");
        insta::assert_snapshot!(name, normalize_encoding_snapshot(&root, &rendered));
    }
}
