use super::symbols_json;
use crate::cli::Format;
use crate::codebase::queries::call_sites::{run_json as call_json, CallSitesArgs};
use crate::codebase::queries::dead_exports::{run_json as dead_json, DeadExportsArgs};
use crate::codebase::symbols::{ExportKindArg, Include, SymbolsArgs, SymbolsMode};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/lang-frontends/language-symbols")
}

struct Case {
    file: &'static str,
    live: &'static str,
    dead: &'static str,
    caller: &'static str,
}

const CASES: &[Case] = &[
    Case {
        file: "python/greeter.py",
        live: "Greeter",
        dead: "Unused",
        caller: "python/caller.py",
    },
    Case {
        file: "go/greet.go",
        live: "Greet",
        dead: "Unused",
        caller: "go/use.go",
    },
    Case {
        file: "rustpkg/greeter.rs",
        live: "Greeter",
        dead: "Unused",
        caller: "rustpkg/lib.rs",
    },
    Case {
        file: "rails/greeter.rb",
        live: "Greeter",
        dead: "Unused",
        caller: "rails/caller.rb",
    },
    Case {
        file: "php/Greeter.php",
        live: "App.Greeter",
        dead: "Unused",
        caller: "php/Caller.php",
    },
    Case {
        file: "java/com/example/Greeter.java",
        live: "Greeter",
        dead: "Unused",
        caller: "java/com/example/Caller.java",
    },
    Case {
        file: "kotlin/com/example/Greeter.kt",
        live: "Greeter",
        dead: "Unused",
        caller: "kotlin/com/example/Caller.kt",
    },
    Case {
        file: "elixir/greeter.ex",
        live: "Greeter",
        dead: "Unused",
        caller: "elixir/caller.ex",
    },
    Case {
        file: "dartapp/lib/greeter.dart",
        live: "Greeter",
        dead: "Unused",
        caller: "dartapp/lib/caller.dart",
    },
    Case {
        file: "swift/Sources/App/Greeter.swift",
        live: "Greeter",
        dead: "Unused",
        caller: "swift/Sources/App/Caller.swift",
    },
    Case {
        file: "dotnet/Greeter.cs",
        live: "Greeter",
        dead: "Unused",
        caller: "dotnet/Caller.cs",
    },
];

#[test]
fn language_symbols_call_sites_and_dead_exports_match_cli_and_napi() {
    let root = fixture_root();
    for case in CASES {
        let symbols = symbols_value(&root, case.file, Vec::new());
        let napi_symbols = napi_symbols(&root, case.file);
        assert_eq!(symbols, napi_symbols, "{}", case.file);
        let export = symbols["files"][0]["exports"]
            .as_array()
            .expect("exports")
            .iter()
            .find(|row| row["name"] == case.live)
            .unwrap_or_else(|| panic!("{} missing {}", case.file, case.live));
        assert_eq!(export["kind"], "declaration");
        let line = export["line"].as_u64().expect("line");
        assert!(line > 0);
        assert!(source_line(&root, case.file, line).contains(last_segment(case.live)));

        let dead = dead_value(&root, case.file, &[case.dead, case.live]);
        assert_eq!(
            dead,
            napi_dead(&root, case.file, &[case.dead, case.live]),
            "{}",
            case.file
        );
        assert_eq!(dead["partial"], false);
        assert_eq!(dead["anyDead"], true);
        let names: Vec<&str> = dead["results"]
            .as_array()
            .expect("results")
            .iter()
            .map(|row| row["name"].as_str().expect("name"))
            .collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted, "{}", case.file);
        let live = result_named(&dead, case.live);
        let dead_row = result_named(&dead, case.dead);
        assert_eq!(live["referenced"], true, "{}", case.file);
        assert_eq!(live["importerCount"], 1, "{}", case.file);
        assert_eq!(dead_row["referenced"], false, "{}", case.file);
        assert_eq!(dead_row["importerCount"], 0);

        let calls = call_value(&root, case.file, case.live);
        assert_eq!(
            calls,
            napi_calls(&root, case.file, case.live),
            "{}",
            case.file
        );
        assert_eq!(calls["partial"], false);
        let sites = calls["callSites"].as_array().expect("sites");
        assert_eq!(sites.len(), 1, "{}", case.file);
        assert_eq!(sites[0]["file"], case.caller);
        assert_eq!(sites[0]["referenceOnly"], true);
        assert_eq!(sites[0]["argCount"], 0);
        assert_eq!(sites[0]["hasSpread"], false);
        assert_eq!(sites[0]["args"], json!([]));
        assert!(sites[0].get("caller").is_none());
        let site_line = sites[0]["line"].as_u64().expect("site line");
        assert!(source_line(&root, case.caller, site_line).contains(last_segment(case.live)));
    }
}

#[test]
fn lowercase_python_calls_are_not_guessed_and_limits_stay_explicit() {
    let root = fixture_root();
    let greet = call_value(&root, "python/greeter.py", "greet");
    assert_eq!(greet["callSites"], json!([]));
    assert!(greet.get("referenceOnly").is_none());

    let filtered = symbols_value(&root, "python/greeter.py", vec![ExportKindArg::Function]);
    assert!(filtered["files"][0].get("exports").is_none());

    let mut impact = symbol_args(&root, "python/greeter.py");
    impact.mode = SymbolsMode::SignatureImpact;
    impact.symbol = Some("Greeter".to_string());
    let error = symbols_json(&impact).unwrap_err().to_string();
    assert!(error.contains("signature-impact"), "{error}");

    let error = dead_json(dead_args(&root, "stray/stray.py", &[]))
        .unwrap_err()
        .to_string();
    assert!(error.contains("stray/stray.py"), "{error}");
    assert!(error.contains("python"), "{error}");
}

fn symbols_value(root: &Path, file: &str, kinds: Vec<ExportKindArg>) -> Value {
    let mut args = symbol_args(root, file);
    args.kinds = kinds;
    parse(symbols_json(&args).unwrap().expect("language symbols"))
}

fn symbol_args(root: &Path, file: &str) -> SymbolsArgs {
    SymbolsArgs {
        files: vec![PathBuf::from(file)],
        root: Some(root.to_path_buf()),
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

fn dead_value(root: &Path, file: &str, names: &[&str]) -> Value {
    parse(dead_json(dead_args(root, file, names)).unwrap())
}

fn dead_args(root: &Path, file: &str, names: &[&str]) -> DeadExportsArgs {
    DeadExportsArgs {
        file: PathBuf::from(file),
        names: names.iter().map(|name| (*name).to_string()).collect(),
        root: Some(root.to_path_buf()),
        tsconfig: None,
        format: Some(Format::Json),
        json: true,
    }
}

fn call_value(root: &Path, file: &str, name: &str) -> Value {
    parse(call_json(call_args(root, file, name)).unwrap())
}

fn call_args(root: &Path, file: &str, name: &str) -> CallSitesArgs {
    CallSitesArgs {
        file: PathBuf::from(file),
        export_name: name.to_string(),
        root: Some(root.to_path_buf()),
        tsconfig: None,
        format: Some(Format::Json),
        json: true,
    }
}

fn napi_symbols(root: &Path, file: &str) -> Value {
    parse(
        crate::napi_api::codebase::symbols_json_impl(json!({
            "root": root.display().to_string(),
            "files": [file],
        }))
        .unwrap(),
    )
}

fn napi_dead(root: &Path, file: &str, names: &[&str]) -> Value {
    parse(
        crate::napi_api::queries::dead_exports_json_impl(json!({
            "root": root.display().to_string(),
            "file": file,
            "names": names,
        }))
        .unwrap(),
    )
}

fn napi_calls(root: &Path, file: &str, name: &str) -> Value {
    parse(
        crate::napi_api::queries::call_sites_json_impl(json!({
            "root": root.display().to_string(),
            "file": file,
            "exportName": name,
        }))
        .unwrap(),
    )
}

fn result_named<'a>(report: &'a Value, name: &str) -> &'a Value {
    report["results"]
        .as_array()
        .expect("results")
        .iter()
        .find(|row| row["name"] == name)
        .unwrap_or_else(|| panic!("missing {name}"))
}

fn source_line(root: &Path, file: &str, line: u64) -> String {
    std::fs::read_to_string(root.join(file))
        .unwrap()
        .lines()
        .nth(line as usize - 1)
        .unwrap_or("")
        .to_string()
}

fn last_segment(name: &str) -> &str {
    name.rsplit(['.', '\\', ':'])
        .find(|part| !part.is_empty())
        .unwrap_or(name)
}

fn parse(text: impl AsRef<str>) -> Value {
    let text = text.as_ref();
    serde_json::from_str(text).unwrap_or_else(|error| panic!("{error}: {text}"))
}
