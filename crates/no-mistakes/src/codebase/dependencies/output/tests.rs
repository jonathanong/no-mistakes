use super::*;
use std::path::{Path, PathBuf};

fn json_value(roots: &[String], entries: &[NodeEntry], root: &Path) -> serde_json::Value {
    let mut buf = Vec::new();
    write_json(roots, entries, root, &mut buf).unwrap();
    serde_json::from_slice(&buf).unwrap()
}

fn p(s: &str) -> PathBuf {
    PathBuf::from(s)
}

fn entry(path: &str, depth: usize) -> NodeEntry {
    NodeEntry {
        node: NodeId::File(p(path)),
        depth,
        via: vec![],
    }
}

fn entry_with_via(path: &str, depth: usize, via: Vec<EdgeKind>) -> NodeEntry {
    NodeEntry {
        node: NodeId::File(p(path)),
        depth,
        via,
    }
}

fn queue_job_entry(queue_file: &str, job: &str, depth: usize) -> NodeEntry {
    NodeEntry {
        node: NodeId::QueueJob {
            queue_file: p(queue_file),
            job: job.to_string(),
        },
        depth,
        via: vec![],
    }
}

fn module_entry(specifier: &str, depth: usize, via: Vec<EdgeKind>) -> NodeEntry {
    NodeEntry {
        node: NodeId::Module(specifier.to_string()),
        depth,
        via,
    }
}

fn symbol_entry(file: &str, symbol: &str, depth: usize, via: Vec<EdgeKind>) -> NodeEntry {
    NodeEntry {
        node: NodeId::Symbol {
            file: p(file),
            symbol: symbol.to_string(),
        },
        depth,
        via,
    }
}

// ── write_json ──────────────────────────────────────────────────────────

#[test]
fn json_empty_entries() {
    let root = p("/root");
    let v = json_value(&["src/a.mts".to_string()], &[], &root);
    assert_eq!(
        v,
        serde_json::json!({
            "roots": ["src/a.mts"],
            "files": [],
        })
    );
}

#[test]
fn json_with_entries_relative_paths() {
    let root = p("/root");
    let entries = vec![entry("/root/src/b.mts", 1), entry("/root/src/c.mts", 2)];
    let v = json_value(&["src/a.mts".to_string()], &entries, &root);
    assert_eq!(
        v,
        serde_json::json!({
            "roots": ["src/a.mts"],
            "files": [
                {"path": "src/b.mts", "depth": 1},
                {"path": "src/c.mts", "depth": 2},
            ],
        })
    );
}

#[test]
fn json_multiple_roots() {
    let root = p("/root");
    let v = json_value(&["a.mts".to_string(), "b.mts".to_string()], &[], &root);
    assert_eq!(
        v,
        serde_json::json!({
            "roots": ["a.mts", "b.mts"],
            "files": [],
        })
    );
}

#[test]
fn json_queue_job_node() {
    let root = p("/root");
    let entries = vec![queue_job_entry("/root/src/queues.mts", "sendWelcome", 1)];
    let v = json_value(&["src/enqueues.mts".to_string()], &entries, &root);
    assert_eq!(
        v,
        serde_json::json!({
            "roots": ["src/enqueues.mts"],
            "files": [
                {"queueFile": "src/queues.mts", "job": "sendWelcome", "depth": 1},
            ],
        })
    );
}

#[test]
fn json_module_node() {
    let root = p("/root");
    let entries = vec![module_entry("@react/client", 1, vec![EdgeKind::Import])];
    let v = json_value(&["src/a.mts".to_string()], &entries, &root);
    assert_eq!(
        v,
        serde_json::json!({
            "roots": ["src/a.mts"],
            "files": [
                {"module": "@react/client", "depth": 1, "via": ["import"]},
            ],
        })
    );
}

#[test]
fn json_symbol_node() {
    let root = p("/root");
    let entries = vec![symbol_entry(
        "/root/src/a.mts",
        "alpha",
        1,
        vec![EdgeKind::Import],
    )];
    let v = json_value(&["src/root.mts".to_string()], &entries, &root);
    assert_eq!(
        v,
        serde_json::json!({
            "roots": ["src/root.mts"],
            "files": [
                {"file": "src/a.mts", "symbol": "alpha", "depth": 1, "via": ["import"]},
            ],
        })
    );
}

// ── write_paths ─────────────────────────────────────────────────────────

#[test]
fn paths_empty_entries() {
    let root = p("/root");
    let mut buf = Vec::new();
    write_paths(&[], &root, &mut buf).unwrap();
    assert!(buf.is_empty());
}

#[test]
fn paths_relative_output() {
    let root = p("/root");
    let entries = vec![entry("/root/src/b.mts", 1), entry("/root/src/c.mts", 2)];
    let mut buf = Vec::new();
    write_paths(&entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert_eq!(s, "src/b.mts\nsrc/c.mts\n");
}

#[test]
fn paths_queue_job_rendered_as_hash() {
    let root = p("/root");
    let entries = vec![queue_job_entry("/root/src/queues.mts", "sendWelcome", 1)];
    let mut buf = Vec::new();
    write_paths(&entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert_eq!(s, "src/queues.mts#sendWelcome\n");
}

#[test]
fn paths_module_rendered_as_specifier() {
    let root = p("/root");
    let entries = vec![module_entry("@react/client", 1, vec![])];
    let mut buf = Vec::new();
    write_paths(&entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert_eq!(s, "@react/client\n");
}

#[test]
fn paths_symbol_rendered_as_file_hash_symbol() {
    let root = p("/root");
    let entries = vec![symbol_entry("/root/src/a.mts", "alpha", 1, vec![])];
    let mut buf = Vec::new();
    write_paths(&entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert_eq!(s, "src/a.mts#alpha\n");
}

// ── write_human ─────────────────────────────────────────────────────────

#[test]
fn human_no_entries() {
    let root = p("/root");
    let mut buf = Vec::new();
    write_human(&["src/a.mts".to_string()], &[], &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("(no results)"));
}

#[test]
fn human_with_entries_indented() {
    let root = p("/root");
    let entries = vec![entry("/root/b.mts", 1), entry("/root/c.mts", 2)];
    let mut buf = Vec::new();
    write_human(&["a.mts".to_string()], &entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("  b.mts"), "depth-1 has 2-space indent");
    assert!(s.contains("    c.mts"), "depth-2 has 4-space indent");
}

#[test]
fn human_queue_job_rendered() {
    let root = p("/root");
    let entries = vec![queue_job_entry("/root/src/queues.mts", "sendWelcome", 1)];
    let mut buf = Vec::new();
    write_human(&["a.mts".to_string()], &entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("src/queues.mts#sendWelcome"));
}

// ── write_md ─────────────────────────────────────────────────────────────

#[test]
fn md_empty_entries() {
    let root = p("/root");
    let mut buf = Vec::new();
    write_md(&["src/a.mts".to_string()], &[], &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("# `src/a.mts`"));
    assert!(s.contains("_No results._"));
}

#[test]
fn md_single_root_with_entries() {
    let root = p("/root");
    let entries = vec![entry("/root/b.mts", 1), entry("/root/c.mts", 2)];
    let mut buf = Vec::new();
    write_md(&["a.mts".to_string()], &entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("# `a.mts`"));
    assert!(s.contains("- `b.mts`"));
    assert!(s.contains("  - `c.mts`")); // depth-2 → 2-space indent
}

#[test]
fn md_multiple_roots() {
    let root = p("/root");
    let mut buf = Vec::new();
    write_md(
        &["a.mts".to_string(), "b.mts".to_string()],
        &[],
        &root,
        &mut buf,
    )
    .unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("# 2 files"));
    assert!(s.contains("- `a.mts`"));
    assert!(s.contains("- `b.mts`"));
}

// ── write_yml ─────────────────────────────────────────────────────────────

#[test]
fn yml_empty_entries() {
    let root = p("/root");
    let mut buf = Vec::new();
    write_yml(&["src/a.mts".to_string()], &[], &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    let v: serde_yaml::Value = serde_yaml::from_str(&s).unwrap();
    assert_eq!(v["roots"].as_sequence().unwrap().len(), 1);
    assert_eq!(v["files"].as_sequence().unwrap().len(), 0);
}

#[test]
fn yml_with_entries() {
    let root = p("/root");
    let entries = vec![entry("/root/src/b.mts", 1)];
    let mut buf = Vec::new();
    write_yml(&["src/a.mts".to_string()], &entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    let v: serde_yaml::Value = serde_yaml::from_str(&s).unwrap();
    let files = v["files"].as_sequence().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["path"].as_str().unwrap(), "src/b.mts");
    assert_eq!(files[0]["depth"].as_u64().unwrap(), 1);
}

#[test]
fn yml_multiple_roots() {
    let root = p("/root");
    let mut buf = Vec::new();
    write_yml(
        &["a.mts".to_string(), "b.mts".to_string()],
        &[],
        &root,
        &mut buf,
    )
    .unwrap();
    let s = String::from_utf8(buf).unwrap();
    let v: serde_yaml::Value = serde_yaml::from_str(&s).unwrap();
    assert_eq!(v["roots"].as_sequence().unwrap().len(), 2);
}

#[test]
fn yml_depth_preserved() {
    let root = p("/root");
    let entries = vec![entry("/root/a.mts", 1), entry("/root/b.mts", 3)];
    let mut buf = Vec::new();
    write_yml(&["root.mts".to_string()], &entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    let v: serde_yaml::Value = serde_yaml::from_str(&s).unwrap();
    let files = v["files"].as_sequence().unwrap();
    assert_eq!(files[1]["depth"].as_u64().unwrap(), 3);
}

#[test]
fn yml_queue_job_node() {
    let root = p("/root");
    let entries = vec![queue_job_entry("/root/src/queues.mts", "sendWelcome", 2)];
    let mut buf = Vec::new();
    write_yml(&["src/enqueues.mts".to_string()], &entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    let v: serde_yaml::Value = serde_yaml::from_str(&s).unwrap();
    let files = v["files"].as_sequence().unwrap();
    assert_eq!(files[0]["queueFile"].as_str().unwrap(), "src/queues.mts");
    assert_eq!(files[0]["job"].as_str().unwrap(), "sendWelcome");
}

// ── via field in JSON/YAML output ────────────────────────────────────────

#[test]
fn json_via_empty_omitted() {
    let root = p("/root");
    let entries = vec![entry("/root/b.mts", 1)];
    let mut buf = Vec::new();
    write_json(&["a.mts".to_string()], &entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    // via is omitted when empty
    assert!(
        v["files"][0].get("via").is_none()
            || v["files"][0]["via"]
                .as_array()
                .map(|a| a.is_empty())
                .unwrap_or(false)
    );
}

#[test]
fn json_via_included_when_present() {
    let root = p("/root");
    let entries = vec![entry_with_via(
        "/root/b.mts",
        1,
        vec![EdgeKind::Import, EdgeKind::TestOf],
    )];
    let mut buf = Vec::new();
    write_json(&["a.mts".to_string()], &entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    let via = v["files"][0]["via"].as_array().unwrap();
    let via_strs: Vec<&str> = via.iter().map(|x| x.as_str().unwrap()).collect();
    assert!(via_strs.contains(&"import"));
    assert!(via_strs.contains(&"test"));
}

#[test]
fn yml_via_included_when_present() {
    let root = p("/root");
    let entries = vec![entry_with_via("/root/b.mts", 1, vec![EdgeKind::RouteRef])];
    let mut buf = Vec::new();
    write_yml(&["a.mts".to_string()], &entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    let v: serde_yaml::Value = serde_yaml::from_str(&s).unwrap();
    let via = v["files"][0]["via"].as_sequence().unwrap();
    assert_eq!(via[0].as_str().unwrap(), "route");
}

#[test]
fn edge_kind_str_all_variants() {
    assert_eq!(EdgeKind::Import.as_str(), "import");
    assert_eq!(EdgeKind::TypeImport.as_str(), "type-import");
    assert_eq!(EdgeKind::DynamicImport.as_str(), "dynamic-import");
    assert_eq!(EdgeKind::Require.as_str(), "require");
    assert_eq!(EdgeKind::TestOf.as_str(), "test");
    assert_eq!(EdgeKind::RouteRef.as_str(), "route");
    assert_eq!(EdgeKind::QueueEnqueue.as_str(), "queue-enqueue");
    assert_eq!(EdgeKind::QueueWorker.as_str(), "queue-worker");
    assert_eq!(EdgeKind::RouteTest.as_str(), "route-test");
    assert_eq!(EdgeKind::Layout.as_str(), "layout");
    assert_eq!(EdgeKind::MarkdownLink.as_str(), "md");
    assert_eq!(EdgeKind::WorkspaceImport.as_str(), "workspace");
    assert_eq!(EdgeKind::PackageDependency.as_str(), "package");
    assert_eq!(EdgeKind::CiInvocation.as_str(), "ci");
    assert_eq!(EdgeKind::HttpCall.as_str(), "http");
    assert_eq!(EdgeKind::ProcessSpawn.as_str(), "process");
    assert_eq!(EdgeKind::AssetImport.as_str(), "asset");
    assert_eq!(EdgeKind::ReactRender.as_str(), "react-render");
    assert_eq!(EdgeKind::Selector.as_str(), "selector");
}

#[test]
fn serialized_edge_kinds_are_documented() {
    let docs = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/graph-edges.md"),
    )
    .unwrap();
    for kind in [
        EdgeKind::Import,
        EdgeKind::TypeImport,
        EdgeKind::DynamicImport,
        EdgeKind::Require,
        EdgeKind::TestOf,
        EdgeKind::RouteRef,
        EdgeKind::QueueEnqueue,
        EdgeKind::QueueWorker,
        EdgeKind::RouteTest,
        EdgeKind::Layout,
        EdgeKind::MarkdownLink,
        EdgeKind::WorkspaceImport,
        EdgeKind::PackageDependency,
        EdgeKind::CiInvocation,
        EdgeKind::HttpCall,
        EdgeKind::ProcessSpawn,
        EdgeKind::AssetImport,
        EdgeKind::ReactRender,
        EdgeKind::Selector,
    ] {
        match kind {
            EdgeKind::Import => {}
            EdgeKind::TypeImport => {}
            EdgeKind::DynamicImport => {}
            EdgeKind::Require => {}
            EdgeKind::TestOf => {}
            EdgeKind::RouteRef => {}
            EdgeKind::QueueEnqueue => {}
            EdgeKind::QueueWorker => {}
            EdgeKind::RouteTest => {}
            EdgeKind::Layout => {}
            EdgeKind::MarkdownLink => {}
            EdgeKind::WorkspaceImport => {}
            EdgeKind::PackageDependency => {}
            EdgeKind::CiInvocation => {}
            EdgeKind::HttpCall => {}
            EdgeKind::ProcessSpawn => {}
            EdgeKind::AssetImport => {}
            EdgeKind::ReactRender => {}
            EdgeKind::Selector => {}
        }
        let serialized = kind.as_str();
        assert!(
            docs.contains(&format!("`{serialized}`")),
            "docs/graph-edges.md must document `{serialized}`"
        );
    }
}
