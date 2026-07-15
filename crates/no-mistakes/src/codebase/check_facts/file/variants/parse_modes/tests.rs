use super::*;
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::ts_source::facts::TsFileFacts;

#[test]
fn mixed_legacy_failure_preserves_standard_diagnostic_and_rejects_symbols() {
    let root = Path::new("/fixture");
    let path = root.join("types.js");
    let plan = CheckFactPlan {
        symbols: true,
        ..CheckFactPlan::default()
    };
    let variant = CheckFactVariant {
        root,
        plan: &plan,
        playwright: None,
    };
    let symbols = Arc::new(crate::codebase::ts_symbols::FileSymbols::default());
    let mut results = vec![Some(CheckFileFacts {
        ts: Arc::new(TsFileFacts {
            symbols: Some(symbols.as_ref().clone()),
            parse_error: Some("standard diagnostic".to_string()),
            ..TsFileFacts::default()
        }),
        symbols: Some(symbols),
        parse_error: Some("standard diagnostic".to_string()),
        ..CheckFileFacts::default()
    })];

    set_legacy_errors(
        &mut results,
        vec![(
            0,
            &variant,
            VariantParseModes {
                standard: true,
                legacy_symbols: true,
            },
        )],
        &path,
        &Arc::from("export interface Shape {}"),
        anyhow::anyhow!("legacy parser panic"),
    );

    let facts = results[0].as_ref().unwrap();
    assert_eq!(facts.parse_error.as_deref(), Some("standard diagnostic"));
    assert_eq!(facts.ts.parse_error.as_deref(), Some("standard diagnostic"));
    assert!(facts.symbols.is_some());
    assert!(facts.ts.symbols.is_some());
    assert!(facts.legacy_symbols.is_none());
    assert_eq!(
        facts.legacy_symbol_parse_error.as_deref(),
        Some("legacy parser panic")
    );
}

#[test]
fn mixed_legacy_success_keeps_standard_symbols_isolated() {
    let standard = Arc::new(crate::codebase::ts_symbols::FileSymbols {
        exports: vec![crate::codebase::ts_symbols::Export {
            name: "standard".to_string(),
            local: None,
            kind: crate::codebase::ts_symbols::ExportKind::Const,
            line: 1,
            is_type_only: false,
        }],
        imports: Vec::new(),
    });
    let legacy = Arc::new(crate::codebase::ts_symbols::FileSymbols {
        exports: vec![crate::codebase::ts_symbols::Export {
            name: "legacy".to_string(),
            local: None,
            kind: crate::codebase::ts_symbols::ExportKind::Interface,
            line: 1,
            is_type_only: true,
        }],
        imports: Vec::new(),
    });
    let mut results = vec![Some(CheckFileFacts {
        ts: Arc::new(TsFileFacts {
            symbols: Some(standard.as_ref().clone()),
            ..TsFileFacts::default()
        }),
        symbols: Some(standard.clone()),
        ..CheckFileFacts::default()
    })];

    merge_legacy_results(&mut results, vec![(0, None, Some(legacy.clone()))]);

    let facts = results[0].as_ref().unwrap();
    assert_eq!(facts.symbols.as_ref(), Some(&standard));
    assert_eq!(facts.ts.symbols.as_ref(), Some(standard.as_ref()));
    assert_eq!(facts.legacy_symbols.as_ref(), Some(&legacy));
}
