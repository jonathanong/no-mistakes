use crate::server_routes::types::Framework;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/ast-snippets/server-routes/fixture")
        .join(name)
}

fn extract_file(path: &std::path::Path) -> anyhow::Result<crate::server_routes::model::FileFacts> {
    let source = std::fs::read_to_string(path)?;
    crate::ast::with_program(path, &source, |program, _| {
        super::extract_program(path, &source, program)
    })
}

#[test]
fn extract_file_covers_import_binding_route_and_mount_shapes() {
    let facts = extract_file(&fixture("extract-walk-all.ts")).unwrap();

    assert!(facts
        .imports
        .iter()
        .any(|import| import.imported == "Router" && import.local == "StringRouter"));
    assert_eq!(facts.exports["publicRouter"], "router");
    assert_eq!(facts.exports["default"], "defaultThing");
    assert_eq!(facts.bindings["api"].framework, Framework::Express);
    assert_eq!(facts.bindings["hono"].prefixes, vec!["/hono"]);
    assert_eq!(facts.bindings["koa"].prefixes, vec!["/koa"]);
    assert_eq!(facts.bindings["loose"].prefixes, vec!["/loose"]);

    let route_pairs: Vec<_> = facts
        .routes
        .iter()
        .map(|route| (route.method.as_str(), route.raw_path.as_str()))
        .collect();
    for expected in [
        ("get", "/direct"),
        ("get", "/namespace-direct"),
        ("delete", "/del"),
        ("get", "/"),
        ("get", "/array"),
        ("get", "/template-array"),
        ("get", "/spread-array"),
        ("get", "/named"),
        ("get", "/root"),
        ("get", "/on"),
        ("delete", "/on"),
        ("get", "/koa-no-prefix"),
        ("get", "/hono-no-prefix"),
        ("get", "/hono-plain"),
        ("get", "/matched"),
        ("get", "/child"),
        ("post", "/post"),
        ("put", "/put"),
        ("get", "/api-server"),
        ("get", "/cjs-express"),
        ("get", "/interop-express"),
        ("get", "/cjs-destructured-router"),
        ("get", "/inline-cjs-express"),
        ("get", "/inline-cjs-router"),
        ("get", "/paren-inline-cjs-express"),
        ("get", "/paren-inline-cjs-router"),
        ("get", "/inline-cjs-hono"),
        ("get", "/inline-cjs-koa"),
        ("get", "/inline-cjs-matched"),
        ("get", "/inline-cjs-api-server"),
        ("get", "/cjs-express-member-router"),
        ("get", "/express-member-alias-router"),
        ("get", "/cjs-hono-member"),
        ("get", "/cjs-hono"),
        ("get", "/cjs-koa"),
        ("get", "/cjs-matched"),
        ("get", "/cjs-api-server"),
        ("get", "/ts-import-equals-express"),
        ("get", "/interop-default-member-router"),
        ("get", "/paren-interop-default-member-router"),
        ("get", "/heuristic"),
        ("get", "/string-module-name-route"),
        ("get", "/destructured-const-string"),
        ("get", "/destructured-bound-api"),
    ] {
        assert!(
            route_pairs.contains(&expected),
            "missing route {expected:?}"
        );
    }
    for skipped in [
        ("get", "/client-supertest-chain"),
        ("get", "/client-supertest-variable"),
        ("get", "/client-axios"),
        ("post", "/client-axios-create"),
        ("get", "/client-got"),
        ("put", "/client-ky"),
        ("get", "/client-superagent"),
        ("get", "/client-playwright"),
        ("get", "/client-axios-static-object"),
        ("get", "/client-node-http"),
        ("get", "/client-const"),
        ("get", "/client-cjs-destructured"),
        ("post", "/client-cjs-nested-destructured"),
        ("get", "/client-imported-destructured"),
        ("post", "/client-created-destructured"),
        ("get", "/client-cjs-created-destructured"),
        ("get", "/client-array-destructured"),
        ("get", "/client-array-rest-destructured"),
        ("get", "/client-cjs-created"),
        ("get", "/client-cjs-factory"),
        ("get", "/client-paren-cjs"),
        ("get", "/client-paren-cjs-created"),
        ("get", "/client-ts-import-equals"),
    ] {
        assert!(
            !route_pairs.contains(&skipped),
            "client request should not be a route definition {skipped:?}"
        );
    }

    assert!(facts
        .mounts
        .iter()
        .any(|mount| mount.parent == "api" && mount.child == "router" && mount.prefix == "/"));
    assert!(facts.mounts.iter().any(|mount| {
        mount.parent == "api" && mount.child == "stringRouter" && mount.prefix == "/api-route"
    }));
}

#[test]
fn default_export_non_identifier_is_ignored() {
    let facts = extract_file(&fixture("default-function.ts")).unwrap();

    assert_eq!(facts.exports["default"], "default");
}

#[test]
fn extract_file_collects_named_handler_query_params() {
    let facts = extract_file(&fixture("named-query-handlers.ts")).unwrap();

    let route_params = |path: &str| {
        facts
            .routes
            .iter()
            .find(|route| route.raw_path == path)
            .map(|route| route.query_params.clone())
            .unwrap_or_else(|| panic!("missing route {path}"))
    };
    assert_eq!(route_params("/search"), vec!["term"]);
    assert_eq!(route_params("/list"), vec!["page"]);
    assert_eq!(route_params("/delegated"), vec!["page"]);
    assert_eq!(route_params("/destructured"), vec!["namedParam"]);
    assert_eq!(route_params("/inline-destructured"), vec!["inlineParam"]);
    assert_eq!(route_params("/exported"), vec!["exported"]);
    assert_eq!(route_params("/exported-function"), vec!["exportedFunction"]);
    assert_eq!(route_params("/defaulted"), vec!["defaulted"]);
}

#[test]
fn extract_file_ignores_named_handlers_without_collectable_bodies() {
    let facts = extract_file(&fixture("named-handler-defensive.ts")).unwrap();

    let declared = facts
        .routes
        .iter()
        .find(|route| route.raw_path == "/declared")
        .expect("declared route");
    assert!(declared.query_params.is_empty());
}

#[test]
fn extract_file_recognizes_fastify_imports_and_verbs() {
    let facts = extract_file(&fixture("fastify-app.ts")).unwrap();

    assert_eq!(facts.bindings["app"].framework, Framework::Fastify);
    assert_eq!(facts.bindings["named"].framework, Framework::Fastify);
    assert_eq!(facts.bindings["ctor"].framework, Framework::Fastify);
    assert_eq!(facts.bindings["equals"].framework, Framework::Fastify);
    assert_eq!(facts.bindings["cjs"].framework, Framework::Fastify);

    let route_pairs: Vec<_> = facts
        .routes
        .iter()
        .map(|route| {
            (
                route.method.as_str(),
                route.raw_path.as_str(),
                route.framework,
            )
        })
        .collect();
    assert!(route_pairs.contains(&("get", "/health", Framework::Fastify)));
    assert!(route_pairs.contains(&("post", "/users", Framework::Fastify)));
    assert!(route_pairs.contains(&("delete", "/named", Framework::Fastify)));
    assert!(route_pairs.contains(&("head", "/ctor", Framework::Fastify)));
    assert!(route_pairs.contains(&("patch", "/equals", Framework::Fastify)));
    assert!(route_pairs.contains(&("put", "/cjs", Framework::Fastify)));
}

#[test]
fn extract_file_recognizes_nestjs_controller_and_verb_decorators() {
    let facts = extract_file(&fixture("nestjs-app.ts")).unwrap();
    assert_eq!(
        facts.bindings["UsersController"].framework,
        Framework::Nestjs
    );
    assert_eq!(facts.bindings["UsersController"].prefixes, vec!["users"]);
    assert_eq!(facts.bindings["HealthController"].prefixes, vec!["health"]);
    let route_pairs: Vec<_> = facts
        .routes
        .iter()
        .map(|route| {
            (
                route.method.as_str(),
                route.raw_path.as_str(),
                route.framework,
            )
        })
        .collect();
    assert!(route_pairs.contains(&("get", "/", Framework::Nestjs)));
    assert!(route_pairs.contains(&("get", "/:id", Framework::Nestjs)));
    assert!(route_pairs.contains(&("post", "/", Framework::Nestjs)));
    assert!(route_pairs.contains(&("head", "/", Framework::Nestjs)));
    assert!(route_pairs.contains(&("put", "/ready", Framework::Nestjs)));
    assert!(route_pairs.contains(&("patch", "/ready", Framework::Nestjs)));
    assert!(route_pairs.contains(&("options", "/ready", Framework::Nestjs)));
    assert!(route_pairs.contains(&("all", "/any", Framework::Nestjs)));
    assert!(route_pairs.contains(&("get", "/host", Framework::Nestjs)));
    assert!(route_pairs.contains(&("delete", "/gone", Framework::Nestjs)));
    assert_eq!(facts.bindings["ExtraController"].prefixes, vec!["extra"]);
    assert!(route_pairs.contains(&("get", "/anon", Framework::Nestjs)));
    assert!(!route_pairs
        .iter()
        .any(|(_, path, _)| path.contains("static")));

    let path = fixture("nestjs-app.ts");
    let map = std::collections::HashMap::from([(path.clone(), facts.clone())]);
    let member = facts
        .routes
        .iter()
        .find(|route| route.binding == "UsersController" && route.raw_path == "/:id")
        .expect("member");
    let expanded: Vec<_> = crate::server_routes::mounts::prefixes_for(member, &map, &[])
        .into_iter()
        .map(|prefix| crate::server_routes::normalize::join_paths(&prefix, &member.raw_path))
        .collect();
    assert_eq!(expanded, vec!["/users/:id".to_string()]);
}

#[test]
fn extract_file_skips_computed_nestjs_paths() {
    let facts = extract_file(&fixture("nestjs-computed.ts")).unwrap();
    assert!(facts.routes.is_empty());
    assert!(!facts.bindings.contains_key("ComputedController"));
}
