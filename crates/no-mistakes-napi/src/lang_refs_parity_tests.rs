use no_mistakes::cli::Format;
use no_mistakes::codebase::queries::call_sites::{run_json as call_json, CallSitesArgs};
use no_mistakes::codebase::queries::dead_exports::{run_json as dead_json, DeadExportsArgs};
use no_mistakes::codebase::symbols::{run_json as symbols_json, Include, SymbolsArgs, SymbolsMode};
use serde_json::{json, Value};
use std::path::PathBuf;

fn parse(text: String) -> Value {
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{error}: {text}"))
}

#[test]
fn language_symbols_call_sites_and_dead_exports_match_cli_and_napi() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lang-frontends/language-symbols");
    let root_str = root.display().to_string();
    let cases = [
        ("python/greeter.py", "Greeter", "Unused"),
        ("go/greet.go", "Greet", "Unused"),
        ("rustpkg/greeter.rs", "Greeter", "Unused"),
        ("rails/greeter.rb", "Greeter", "Unused"),
        ("php/Greeter.php", "App.Greeter", "Unused"),
        ("java/com/example/Greeter.java", "Greeter", "Unused"),
        ("kotlin/com/example/Greeter.kt", "Greeter", "Unused"),
        ("elixir/greeter.ex", "Greeter", "Unused"),
        ("dartapp/lib/greeter.dart", "Greeter", "Unused"),
        ("swift/Sources/App/Greeter.swift", "Greeter", "Unused"),
        ("dotnet/Greeter.cs", "Greeter", "Unused"),
    ];
    for (file, live, dead_name) in cases {
        let cli_symbols = parse(
            symbols_json(SymbolsArgs {
                files: vec![PathBuf::from(file)],
                root: Some(root.clone()),
                tsconfig: None,
                config: None,
                mode: SymbolsMode::List,
                symbol: None,
                kinds: Vec::new(),
                include: Include::Exports,
                format: Some(Format::Json),
                json: true,
                timings: false,
            })
            .unwrap(),
        );
        let napi_symbols = parse(
            crate::codebase::symbols_json_impl(json!({ "root": &root_str, "files": [file] }))
                .unwrap(),
        );
        assert_eq!(cli_symbols, napi_symbols, "{file}");

        let cli_dead = parse(
            dead_json(DeadExportsArgs {
                file: PathBuf::from(file),
                names: vec![dead_name.to_string(), live.to_string()],
                root: Some(root.clone()),
                tsconfig: None,
                format: Some(Format::Json),
                json: true,
            })
            .unwrap(),
        );
        let napi_dead = parse(
            crate::queries::dead_exports_json_impl(json!({
                "root": &root_str,
                "file": file,
                "names": [dead_name, live],
            }))
            .unwrap(),
        );
        assert_eq!(cli_dead, napi_dead, "{file}");

        let cli_calls = parse(
            call_json(CallSitesArgs {
                file: PathBuf::from(file),
                export_name: live.to_string(),
                root: Some(root.clone()),
                tsconfig: None,
                format: Some(Format::Json),
                json: true,
            })
            .unwrap(),
        );
        let napi_calls = parse(
            crate::queries::call_sites_json_impl(json!({
                "root": &root_str,
                "file": file,
                "exportName": live,
            }))
            .unwrap(),
        );
        assert_eq!(cli_calls, napi_calls, "{file}");
    }
}

#[test]
fn unreadable_configured_caller_matches_dead_exports_napi() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/unreadable-language-caller");
    let dead = parse(
        dead_json(DeadExportsArgs {
            file: PathBuf::from("python/greeter.py"),
            names: vec!["Greeter".to_string(), "Unused".to_string()],
            root: Some(root.clone()),
            tsconfig: None,
            format: Some(Format::Json),
            json: true,
        })
        .unwrap(),
    );
    let napi_dead = parse(
        crate::queries::dead_exports_json_impl(json!({
            "root": root.display().to_string(),
            "file": "python/greeter.py",
            "names": ["Greeter", "Unused"],
        }))
        .unwrap(),
    );
    assert_eq!(dead, napi_dead);
}
