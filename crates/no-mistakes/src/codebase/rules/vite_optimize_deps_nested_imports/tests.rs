use super::declared::dependency_names;
use super::entry_imports::collect;
use super::specifier::{is_node_builtin, nested_pair, package_name};
use std::path::Path;

#[test]
fn package_name_keeps_the_scope_and_drops_a_subpath() {
    assert_eq!(
        package_name("@widgets/ui/theme").as_deref(),
        Some("@widgets/ui")
    );
    assert_eq!(package_name("items/shape").as_deref(), Some("items"));
    assert_eq!(package_name("./local"), None);
    assert_eq!(package_name("node:fs"), None);
    assert_eq!(package_name("node:fs/promises"), None);
    assert_eq!(package_name("path"), None);
    assert_eq!(package_name("fs"), None);
    assert_eq!(package_name("crypto"), None);
    assert_eq!(package_name("fs/promises"), None);
    assert_eq!(package_name("path/posix"), None);
    assert_eq!(package_name("@widgets"), None);
    assert_eq!(package_name("@/ui"), None);
    assert_eq!(package_name("@widgets/"), None);
}

#[test]
fn node_builtins_include_bare_names_and_subpaths() {
    assert!(is_node_builtin("fs"));
    assert!(is_node_builtin("path"));
    assert!(is_node_builtin("crypto"));
    assert!(is_node_builtin("fs/promises"));
    assert!(is_node_builtin("node:fs"));
    assert!(is_node_builtin("node:fs/promises"));
    assert!(!is_node_builtin("items"));
    assert!(!is_node_builtin("widgets/path"));
}

#[test]
fn collect_skips_builtins_and_the_package_name() {
    let source = "\
import path from 'path';
import { readFileSync } from 'fs';
import { readFile } from 'fs/promises';
import nodeFs from 'node:fs';
import widgets from 'widgets';
import { shape } from 'items';
export * from 'stream/promises';
export { readFileSync as read } from 'fs';
const dynamic = import('node:path');
const crypto = require('crypto');
void dynamic;
void nodeFs;
void path;
void readFile;
void widgets;
void read;
void crypto;
void shape;
";
    let imports = collect(Path::new("index.ts"), source, "widgets").unwrap();
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].package, "items");
    assert_eq!(imports[0].line, 6);

    let scoped = "\
import theme from '@widgets/ui/theme';
import { shape } from 'items';
";
    let imports = collect(Path::new("index.ts"), scoped, "@widgets/ui").unwrap();
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].package, "items");
}

#[test]
fn dependency_names_counts_optional_and_bundled_declarations() {
    let value = serde_json::json!({
        "dependencies": { "items": "1.0.0", "path": "1.0.0" },
        "optionalDependencies": { "orders": "1.0.0" },
        "peerDependencies": null,
        "bundledDependencies": ["labels", "./local", 1],
        "bundleDependencies": ["widgets"],
        "devDependencies": { "extra": "1.0.0" }
    });
    let names = dependency_names(&value).unwrap();
    assert!(names.contains("items"));
    assert!(names.contains("orders"));
    assert!(names.contains("labels"));
    assert!(names.contains("widgets"));
    assert!(!names.contains("path"));
    assert!(!names.contains("extra"));
    assert!(!names.contains("local"));
}

#[test]
fn dependency_names_rejects_a_non_object_and_a_non_array() {
    let optional = serde_json::json!({ "optionalDependencies": [] });
    let error = dependency_names(&optional).unwrap_err();
    assert!(error.contains("optionalDependencies must be an object"));

    let bundled = serde_json::json!({ "bundledDependencies": {} });
    let error = dependency_names(&bundled).unwrap_err();
    assert!(error.contains("bundledDependencies must be an array"));

    let alias = serde_json::json!({ "bundleDependencies": "widgets" });
    let error = dependency_names(&alias).unwrap_err();
    assert!(error.contains("bundleDependencies must be an array"));
}

#[test]
fn nested_pair_accepts_only_one_separator() {
    assert_eq!(
        nested_pair(" widgets > items "),
        Some(("widgets".to_string(), "items".to_string()))
    );
    assert_eq!(nested_pair("react"), None);
    assert_eq!(nested_pair("widgets > items > extra"), None);
}
