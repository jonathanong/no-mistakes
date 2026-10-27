use super::*;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ObjectPropertyKind, PropertyKey, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::fs;
use std::path::PathBuf;

fn spawn_case(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/ast-snippets/ts-process-spawn")
            .join(name),
    )
}

fn fixture_source(name: &str) -> (PathBuf, String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/ast-snippets/ts-process-spawn/fixture/project")
        .join(name);
    let source = fs::read_to_string(&path).expect("fixture source must be readable");
    (path, source)
}

#[test]
fn pass4a_ignored_local_spawn_target_does_not_shadow_visible_root_fallback() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4a-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let spawner = root.join("process/nested/spawn.ts");
    let source = fs::read_to_string(&spawner).unwrap();
    let visible_files: std::collections::HashSet<PathBuf> =
        crate::codebase::ts_source::discover_visible_paths(&root)
            .into_iter()
            .collect();

    let edges = extract_spawn_edges_from_visible(&source, &spawner, &root, &visible_files);

    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].entry, root.join("worker.ts"));
}

#[test]
fn helper_branches_visit_present_optional_values() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/ast-snippets/ts-process-spawn/fixture/project");
    let (config_path, source) = fixture_source("configs/spawn-all.tsx");
    let allocator = Allocator::default();
    let program = Parser::new(&allocator, &source, SourceType::tsx())
        .parse()
        .program;

    let mut edges = Vec::new();
    let var_decl = program
        .body
        .iter()
        .find_map(|stmt| match stmt {
            Statement::VariableDeclaration(var_decl) => Some(var_decl),
            _ => None,
        })
        .expect("fixture must contain variable declarations");
    collect_from_optional_expr(
        var_decl.declarations[0].init.as_ref(),
        &source,
        &config_path,
        &root,
        None,
        &mut edges,
    );

    let config_object = program
        .body
        .iter()
        .filter_map(|stmt| match stmt {
            Statement::VariableDeclaration(var_decl) => var_decl.declarations.first(),
            _ => None,
        })
        .filter_map(|decl| decl.init.as_ref())
        .find(|expr| matches!(expr, Expression::ObjectExpression(_)))
        .expect("fixture must contain an object expression");
    extract_optional_web_server_entry(Some(config_object), &config_path, &root, None, &mut edges);

    let command_expr = match config_object {
        Expression::ObjectExpression(obj) => obj.properties.iter().find_map(|prop| {
            let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                return None;
            };
            let PropertyKey::StaticIdentifier(id) = &prop.key else {
                return None;
            };
            if id.name.as_str() != "webServer" {
                return None;
            }
            let Expression::ArrayExpression(array) = &prop.value else {
                return None;
            };
            let first = array.elements.first()?.as_expression()?;
            let Expression::ObjectExpression(first_obj) = first else {
                return None;
            };
            first_obj.properties.iter().find_map(|entry| {
                let ObjectPropertyKind::ObjectProperty(entry) = entry else {
                    return None;
                };
                let PropertyKey::StaticIdentifier(key) = &entry.key else {
                    return None;
                };
                (key.name.as_str() == "command").then_some(&entry.value)
            })
        }),
        _ => None,
    };
    let mut cwd = None;
    if let Some(expr) = command_expr {
        assign_literal_cwd(&mut cwd, expr);
    }
    assert!(cwd.is_some());
}

#[test]
fn playwright_webserver_string_literal() {
    let root = spawn_case("playwright-webserver-string-literal");
    let config_path = root.join("playwright.config.mts");

    let src = r#"
export default defineConfig({
  webServer: [
    {
      command: 'NODE_ENV=test node backend/server/serve.mts',
      name: 'backend',
    },
  ],
})
"#;

    let edges = extract_spawn_edges(src, &config_path, &root);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].entry, root.join("backend/server/serve.mts"));
    assert_eq!(edges[0].spawner, config_path);
}

#[test]
fn playwright_webserver_string_literal_env_prefix() {
    let root = spawn_case("playwright-webserver-string-literal-env-prefix");
    let config_path = root.join("playwright.config.mts");

    let src = r#"
export default defineConfig({
  webServer: [
    {
      command: 'IMAGE_LAMBDA_PORT= node lambdas/dev-server.mts',
    },
  ],
})
"#;

    let edges = extract_spawn_edges(src, &config_path, &root);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].entry, root.join("lambdas/dev-server.mts"));
}

#[test]
fn playwright_webserver_template_literal_with_interpolation() {
    // Template literal commands: quasis are concatenated, interpolations replaced with "".
    // `IMAGE_LAMBDA_PORT=${process.env.PORT} node lambdas/dev-server.mts`
    // → quasis joined: "IMAGE_LAMBDA_PORT=" + " node lambdas/dev-server.mts"
    // → tokenized: skip "IMAGE_LAMBDA_PORT=", skip "node", resolve "lambdas/dev-server.mts"
    let root = spawn_case("playwright-webserver-template-literal-with-interpolation");
    let config_path = root.join("playwright.config.mts");

    let src = r#"
export default defineConfig({
  webServer: [
    {
      command: `IMAGE_LAMBDA_PORT=${process.env.PORT} node lambdas/dev-server.mts`,
    },
  ],
})
"#;

    let edges = extract_spawn_edges(src, &config_path, &root);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].entry, root.join("lambdas/dev-server.mts"));
}

#[test]
fn playwright_webserver_multiple_entries() {
    let root = spawn_case("playwright-webserver-multiple-entries");
    let config_path = root.join("playwright.config.mts");

    let src = r#"
export default defineConfig({
  webServer: [
    { command: 'NODE_ENV=test node backend/server/serve.mts' },
    { command: 'node cloudflare-worker/scripts/start-wrangler.mts' },
  ],
})
"#;

    let edges = extract_spawn_edges(src, &config_path, &root);
    assert_eq!(edges.len(), 2);
    let entries: Vec<_> = edges.iter().map(|e| e.entry.clone()).collect();
    assert!(entries.contains(&root.join("backend/server/serve.mts")));
    assert!(entries.contains(&root.join("cloudflare-worker/scripts/start-wrangler.mts")));
}

#[test]
fn spawn_call_with_literal_cmd() {
    let root = spawn_case("spawn-call-with-literal-cmd");
    let caller = root.join("test/setup.mts");

    let src = r#"spawn('node', ['scripts/runner.mts'], { cwd: undefined })"#;
    let edges = extract_spawn_edges(src, &caller, &root);
    // The cmd is 'node' which is a runtime prefix, not a file path.
    // The spawn call resolves the first arg, which is 'node' — not a file path.
    // So no edges (the file is in args[1], which is an array, not directly supported).
    // This tests that we don't crash; actual file-in-args case is a future enhancement.
    assert!(
        edges.is_empty(),
        "file paths in spawn args arrays are not yet supported and must produce no edge"
    );
}

#[test]
fn exec_call_resolves_shell_entry() {
    let root = spawn_case("exec-call-resolves-shell-entry");
    let caller = root.join("setup.mts");

    let src = r#"exec('NODE_ENV=test tsx scripts/migrate.mts')"#;
    let edges = extract_spawn_edges(src, &caller, &root);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].entry, root.join("scripts/migrate.mts"));
}

#[test]
fn skips_nonexistent_files() {
    let root = spawn_case("skips-nonexistent-files");
    let caller = root.join("setup.mts");

    let src = r#"exec('node scripts/does-not-exist.mts')"#;
    let edges = extract_spawn_edges(src, &caller, &root);
    assert!(edges.is_empty(), "nonexistent file must produce no edge");
}

#[test]
fn skips_ternary_command() {
    let root = spawn_case("skips-ternary-command");
    let config_path = root.join("playwright.config.mts");

    // Ternary: CI ? 'node server.js' : 'npm run dev' — not a literal, must be skipped
    let src = r#"
export default defineConfig({
  webServer: [
    { command: CI ? 'node server.js' : 'npm run dev', cwd: 'web' },
  ],
})
"#;

    let edges = extract_spawn_edges(src, &config_path, &root);
    assert!(edges.is_empty(), "conditional command must be skipped");
}

#[test]
fn ignores_unresolved_and_nonliteral_spawn_shapes() {
    let root = spawn_case("ignores-unresolved-and-nonliteral-spawn-shapes");
    let caller = root.join("setup.mts");

    let src = r#"
function emptyBody();
export default function alsoEmpty();
spawn("scripts/missing.mts");
exec("node scripts/missing.mts");
defineConfig({ webServer: [{ command: dynamic }, "node scripts/existing.mts", { ...other }] });
exec("node scripts/missing.mts", { env: {} });
exec("node http://example.com/script.mts");
"#;
    let edges = extract_spawn_edges(src, &caller, &root);
    assert!(edges.is_empty());
}

#[test]
fn absolute_cwd_resolves_spawn_entry() {
    // The temp path contains an apostrophe, which the JS literal must encode.
    let root = crate::test_support::materialize_quote_containing_fixture(&spawn_case(
        "absolute-cwd-resolves-spawn-entry",
    ));
    let caller = root.path().join("setup.mts");
    let cwd = root.path().join("apps/site");
    let src = format!(
        "spawn('scripts/from-abs.mts', [], {{ cwd: {} }})",
        crate::test_support::js_string_literal(cwd.to_string_lossy())
    );

    let edges = extract_spawn_edges(&src, &caller, root.path());
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].entry, cwd.join("scripts/from-abs.mts"));
}

#[test]
fn fixture_spawn_walker_covers_statement_expression_and_resolution_shapes() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/ast-snippets/ts-process-spawn/fixture/project"),
    );
    let config_path = root.join("configs/spawn-all.tsx");
    let source = std::fs::read_to_string(&config_path).expect("spawn fixture must be readable");
    let edges = extract_spawn_edges(&source, &config_path, &root);
    let entries: Vec<String> = edges
        .iter()
        .map(|edge| {
            edge.entry
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    for expected in [
        "scripts/root.mts",
        "scripts/exec-file.mts",
        "scripts/fork.mts",
        "scripts/direct-spawn.mts",
        "apps/site/scripts/spawn.mts",
        "apps/site/scripts/web.mts",
        "apps/site/scripts/template-web.mts",
        "scripts/define-config.mts",
        "scripts/export-default-config.mts",
        "scripts/block.mts",
        "scripts/function.mts",
        "scripts/export-var.mts",
        "scripts/export-function.mts",
        "scripts/default-function.mts",
        "scripts/default-arrow.mts",
        "scripts/if.mts",
        "scripts/else.mts",
        "scripts/try.mts",
        "scripts/catch.mts",
        "scripts/finally.mts",
        "scripts/while.mts",
        "scripts/for.mts",
        "scripts/for-in.mts",
        "scripts/for-of.mts",
        "scripts/await.mts",
        "scripts/member-exec.mts",
        "scripts/nested.mts",
    ] {
        assert!(
            entries.iter().any(|entry| entry == expected),
            "{expected} missing from {entries:?}"
        );
    }
    assert!(!entries.iter().any(|entry| entry.contains("missing")));
}
