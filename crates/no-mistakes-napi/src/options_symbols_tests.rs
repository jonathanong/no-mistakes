use super::*;

#[test]
fn parse_symbols_mode_defaults_and_rejects_unknown_modes() {
    assert_eq!(parse_symbols_mode(None).unwrap(), SymbolsMode::List);
    assert_eq!(
        parse_symbols_mode(Some("signature-impact")).unwrap(),
        SymbolsMode::SignatureImpact
    );

    let err = parse_symbols_mode(Some("unknown")).unwrap_err();
    assert!(err.to_string().contains("unknown symbols mode: unknown"));
}
