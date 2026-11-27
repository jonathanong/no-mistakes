use super::symbols_json;
use crate::cli::Format;
use crate::codebase::dependencies::graph::{
    proven_dotnet_reference_files, proven_lang_reference_files, proven_swift_reference_files,
    EdgeKind,
};
use crate::codebase::dotnet::{DotnetFactMap, DotnetFileFacts};
use crate::codebase::lang_frontends::{LangFactMap, LangFileFacts};
use crate::codebase::queries::call_sites::{run as call_run, run_json as call_json, CallSitesArgs};
use crate::codebase::queries::dead_exports::{
    run as dead_run, run_json as dead_json, DeadExportsArgs,
};
use crate::codebase::swift::{SwiftFactMap, SwiftFileFacts};
use crate::codebase::symbols::{Include, SymbolsArgs, SymbolsMode};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/lang-frontends/language-symbols")
}

fn symbol_args(file: &str) -> SymbolsArgs {
    SymbolsArgs {
        files: vec![PathBuf::from(file)],
        root: Some(root()),
        tsconfig: None,
        config: None,
        mode: SymbolsMode::List,
        symbol: None,
        kinds: Vec::new(),
        include: Include::Exports,
        format: Some(Format::Json),
        json: true,
        timings: false,
    }
}

fn dead_args(file: &str) -> DeadExportsArgs {
    DeadExportsArgs {
        file: PathBuf::from(file),
        names: Vec::new(),
        root: Some(root()),
        tsconfig: None,
        format: Some(Format::Json),
        json: true,
    }
}

fn error_text<T>(result: anyhow::Result<T>) -> String {
    match result {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected an error"),
    }
}

fn call_args(file: &str, name: &str) -> CallSitesArgs {
    CallSitesArgs {
        file: PathBuf::from(file),
        export_name: name.to_string(),
        root: Some(root()),
        tsconfig: None,
        format: Some(Format::Json),
        json: true,
    }
}

#[test]
fn empty_name_lists_use_declarations_and_unknown_names_fail() {
    let text = dead_json(dead_args("python/greeter.py")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let names: Vec<&str> = value["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"Greeter")
            && names.contains(&"Unused")
            && value["anyDead"].as_bool() == Some(true),
        "{value}"
    );
    let error = call_json(call_args("python/greeter.py", "NoSuch"))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("NoSuch") && error.contains("not a declaration"),
        "{error}"
    );
    assert!(call_run(call_args("python/greeter.py", "NoSuch")).is_err());
    assert!(dead_run(dead_args("python/missing.py")).is_err());
}

#[test]
fn non_language_symbols_are_absent_and_unconfigured_files_fail() {
    assert!(symbols_json(&symbol_args("swift/Package.swift"))
        .unwrap()
        .is_none());
    assert!(symbols_json(&symbol_args("readme.md")).unwrap().is_none());

    let config = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/importers/tests-scoped-alias/.no-mistakes.yml");
    let unconfigured = error_text(super::load::prepare_file(
        Path::new("python/greeter.py"),
        Some(&root()),
        Some(&config),
    ));
    assert!(
        unconfigured.contains("not in a configured") && unconfigured.contains("python"),
        "{unconfigured}"
    );

    let missing_config = error_text(super::load::prepare_file(
        Path::new("python/greeter.py"),
        Some(&root()),
        Some(&root().join("missing.yml")),
    ));
    assert!(
        missing_config.contains("loading .no-mistakes.yml"),
        "{missing_config}"
    );
    assert!(
        super::load::prepare_file(Path::new("python/missing.py"), Some(&root()), None).is_err()
    );
    assert!(super::language_call_rows(
        Path::new("python/missing.py"),
        "Greeter",
        Some(&root()),
        None
    )
    .is_err());
    assert!(call_json(call_args("missing.ts", "Greeter")).is_err());
    assert!(dead_json(dead_args("missing.ts")).is_err());
}

#[test]
fn files_outside_configured_native_packages_are_rejected() {
    // Package lists are resolved from this root, so the real sources fall outside them.
    let config = root().join(".no-mistakes.yml");
    let swift = error_text(super::load::prepare_file(
        Path::new("Greeter.swift"),
        Some(&root().join("swift/Sources/App")),
        Some(&config),
    ));
    let dotnet = error_text(super::load::prepare_file(
        Path::new("Greeter.cs"),
        Some(&root().join("dotnet")),
        Some(&config),
    ));
    assert!(
        swift.contains("outside the configured") && swift.contains("swift"),
        "{swift}"
    );
    assert!(
        dotnet.contains("outside the configured") && dotnet.contains("dotnet"),
        "{dotnet}"
    );
}

#[test]
fn reference_queries_stop_when_the_declaration_is_not_linked() {
    let declaration = PathBuf::from("decl.py");
    let mut lang = LangFactMap::default();
    assert!(proven_lang_reference_files(
        &lang,
        &declaration,
        "Greeter",
        EdgeKind::PythonImport,
        EdgeKind::PythonReference,
    )
    .is_empty());
    lang.files.insert(
        declaration.clone(),
        LangFileFacts {
            path: declaration.clone(),
            ..LangFileFacts::default()
        },
    );
    assert!(proven_lang_reference_files(
        &lang,
        &declaration,
        "Greeter",
        EdgeKind::PythonImport,
        EdgeKind::PythonReference,
    )
    .is_empty());
    lang.declarations
        .insert("Greeter".to_string(), BTreeSet::from([declaration.clone()]));
    assert!(proven_lang_reference_files(
        &lang,
        &declaration,
        "Greeter",
        EdgeKind::PythonImport,
        EdgeKind::PythonReference,
    )
    .is_empty());

    let mut swift = SwiftFactMap::default();
    assert!(proven_swift_reference_files(&swift, &declaration, "Greeter").is_empty());
    swift
        .declarations
        .insert("Greeter".to_string(), BTreeSet::from([declaration.clone()]));
    swift.files.insert(
        declaration.clone(),
        SwiftFileFacts {
            path: declaration.clone(),
            references: vec!["Greeter".to_string()],
            ..SwiftFileFacts::default()
        },
    );
    assert!(proven_swift_reference_files(&swift, &declaration, "Greeter").is_empty());

    let mut dotnet = DotnetFactMap::default();
    assert!(proven_dotnet_reference_files(&dotnet, &declaration, "Greeter").is_empty());
    dotnet
        .declarations
        .insert("Greeter".to_string(), BTreeSet::from([declaration.clone()]));
    dotnet.files.insert(
        declaration.clone(),
        DotnetFileFacts {
            path: declaration.clone(),
            references: vec!["Greeter".to_string()],
            ..DotnetFileFacts::default()
        },
    );
    assert!(proven_dotnet_reference_files(&dotnet, &declaration, "Greeter").is_empty());
}
