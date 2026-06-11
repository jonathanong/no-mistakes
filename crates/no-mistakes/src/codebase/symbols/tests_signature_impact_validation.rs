#[test]
fn signature_impact_validates_symbol_and_file_count() {
    let mut missing_symbol = impact_args("parseDate", Format::Json);
    missing_symbol.symbol = None;
    let err = impact::collect_report(&missing_symbol).unwrap_err();
    assert!(err.to_string().contains("requires --symbol"));

    let mut multiple_files = impact_args("parseDate", Format::Json);
    multiple_files.files.push(PathBuf::from("other.mts"));
    let err = impact::collect_report(&multiple_files).unwrap_err();
    assert!(err.to_string().contains("exactly one file"));

    let err = impact::collect_report(&impact_args("missing", Format::Json)).unwrap_err();
    assert!(err.to_string().contains("is not exported"));

    let err =
        impact::collect_report(&impact_file_args("unused-import.mts", "parseDate", Format::Json))
            .unwrap_err();
    assert!(err.to_string().contains("is not exported"));
}
