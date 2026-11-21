use super::include_config::{parse_vite_config, ParsedConfig};
use super::include_source::{parse_include_source, SourceList};
use std::path::Path;

fn include_texts(source: &str, export_name: &str) -> Vec<String> {
    match parse_include_source(Path::new("listed.ts"), source, export_name) {
        SourceList::Includes(entries) => entries.into_iter().map(|entry| entry.text).collect(),
        SourceList::Unanalyzable { reason, .. } => panic!("{source}: {reason}"),
    }
}

fn unanalyzable(source: &str, export_name: &str) -> String {
    match parse_include_source(Path::new("listed.ts"), source, export_name) {
        SourceList::Includes(_) => panic!("expected an unanalyzable include source"),
        SourceList::Unanalyzable { reason, .. } => reason,
    }
}

#[test]
fn export_braces_read_a_local_const_array() {
    let same_name = "\
const listed = ['widgets > items', 'widgets > orders'] as const;
export { listed };
";
    assert_eq!(
        include_texts(same_name, "listed"),
        vec!["widgets > items", "widgets > orders"]
    );
    let alias = "\
const nested = ['widgets > items'] satisfies readonly string[];
export { nested as listed };
";
    assert_eq!(include_texts(alias, "listed"), vec!["widgets > items"]);
    let exported_const = "\
export const nested = ['widgets > orders'] as const;
export { nested as listed };
";
    assert_eq!(
        include_texts(exported_const, "listed"),
        vec!["widgets > orders"]
    );
}

#[test]
fn export_braces_skip_unrelated_names_and_non_const_exports() {
    let source = "\
const hidden = ['widgets > orders'];
const extra = ['widgets > labels'];
const skipped = ['widgets > ignored'], listed = ['widgets > items'];
export function shape() { return 1; }
export const other = ['widgets > orders'];
export { type hidden, extra as otherName, listed };
";
    assert_eq!(include_texts(source, "listed"), vec!["widgets > items"]);
    assert!(unanalyzable(
        "const listed = ['widgets > items'];\nexport { type listed };\n",
        "listed"
    )
    .contains("does not export const"));
}

#[test]
fn export_braces_reject_indirect_and_non_const_bindings() {
    assert!(
        unanalyzable("export { listed } from './other';\n", "listed")
            .contains("does not export const")
    );
    assert!(unanalyzable(
        "const listed = ['widgets > items'];\nexport type { listed };\n",
        "listed"
    )
    .contains("does not export const"));
    assert!(unanalyzable(
        "let listed = ['widgets > items'];\nexport { listed };\n",
        "listed"
    )
    .contains("does not export const"));
    assert!(
        unanalyzable("const listed = names;\nexport { listed };\n", "listed")
            .contains("not a static array of strings")
    );
}

#[test]
fn include_sources_ignore_a_plugin_default_and_keep_a_static_include_key() {
    let plugin = "\
export default function widgetsPlugin() {
  return { name: 'widgets', configResolved() {} };
}
";
    match parse_vite_config(Path::new("plugin.config.ts"), plugin, true) {
        ParsedConfig::Includes(entries) => assert!(entries.is_empty()),
        ParsedConfig::Unanalyzable { reason, .. } => panic!("{reason}"),
    }
    match parse_vite_config(Path::new("plugin.config.ts"), plugin, false) {
        ParsedConfig::Unanalyzable { reason, .. } => {
            assert!(reason.contains("no static default export"));
        }
        ParsedConfig::Includes(_) => {
            panic!("plugin config stays unanalyzable without includeSources")
        }
    }
    let spread = "\
const extra = {};
export default { optimizeDeps: { ...extra, include: ['widgets > items'] } };
";
    match parse_vite_config(Path::new("vite.config.ts"), spread, true) {
        ParsedConfig::Includes(entries) => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].text, "widgets > items");
        }
        ParsedConfig::Unanalyzable { reason, .. } => panic!("{reason}"),
    }
    match parse_vite_config(Path::new("vite.config.ts"), spread, false) {
        ParsedConfig::Unanalyzable { reason, .. } => {
            assert!(reason.contains("not a static object"));
        }
        ParsedConfig::Includes(_) => panic!("optimizeDeps spread stays unanalyzable"),
    }
}
