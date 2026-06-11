use super::super::*;
use std::collections::HashMap;

#[test]
fn summarizes_basic_route_helper_patterns() {
    let source = r#"
export function entityHref(entity: { id: string }, kind: string): string {
  return `/prefix/${entity.id}/suffix/${kind}`;
}
"#;
    let facts = extract_route_ref_facts(source, "links.ts");
    assert_eq!(facts.route_helpers.len(), 1);
    assert_eq!(facts.route_helpers[0].name, "entityHref");
    assert_eq!(facts.route_helpers[0].patterns, vec!["/prefix/*/suffix/*"]);
}

#[test]
fn summarizes_default_exported_route_helper_patterns() {
    let named = r#"
export default function entityHref(entity: { id: string }): string {
  return `/entities/${entity.id}`;
}
"#;
    let facts = extract_route_ref_facts(named, "entity-href.ts");
    let helpers = facts
        .route_helpers
        .iter()
        .map(|helper| (helper.name.as_str(), helper.patterns.clone()))
        .collect::<HashMap<_, _>>();
    assert_eq!(
        helpers.get("default"),
        Some(&vec!["/entities/*".to_string()])
    );
    assert_eq!(
        helpers.get("entityHref"),
        Some(&vec!["/entities/*".to_string()])
    );

    let anonymous = r#"
export default (entity: { id: string }) => `/anonymous/${entity.id}`;
"#;
    let facts = extract_route_ref_facts(anonymous, "anonymous-href.ts");
    assert_eq!(facts.route_helpers.len(), 1);
    assert_eq!(facts.route_helpers[0].name, "default");
    assert_eq!(facts.route_helpers[0].patterns, vec!["/anonymous/*"]);

    let function_expression = r#"
export default function (entity: { id: string }): string {
  return `/function-expression/${entity.id}`;
}
"#;
    let facts = extract_route_ref_facts(function_expression, "function-expression-href.ts");
    assert_eq!(facts.route_helpers.len(), 1);
    assert_eq!(facts.route_helpers[0].name, "default");
    assert_eq!(
        facts.route_helpers[0].patterns,
        vec!["/function-expression/*"]
    );

    let declaration = r#"
export default function declared(entity: { id: string }): string;
"#;
    let facts = extract_route_ref_facts(declaration, "declaration-href.ts");
    assert!(facts.route_helpers.is_empty());

    let parenthesized_function_expression = r#"
export default (function (entity: { id: string }): string {
  return `/parenthesized-function/${entity.id}`;
});
"#;
    let facts = extract_route_ref_facts(
        parenthesized_function_expression,
        "parenthesized-function-href.ts",
    );
    assert_eq!(facts.route_helpers.len(), 1);
    assert_eq!(facts.route_helpers[0].name, "default");
    assert_eq!(
        facts.route_helpers[0].patterns,
        vec!["/parenthesized-function/*"]
    );

    let nested_parenthesized_function_expression = r#"
export default ((function (entity: { id: string }): string {
  return `/nested-parenthesized-function/${entity.id}`;
}));
"#;
    let facts = extract_route_ref_facts(
        nested_parenthesized_function_expression,
        "nested-parenthesized-function-href.ts",
    );
    assert_eq!(facts.route_helpers.len(), 1);
    assert_eq!(
        facts.route_helpers[0].patterns,
        vec!["/nested-parenthesized-function/*"]
    );

    let non_helper_expression = "export default ({ href: '/ignored' });";
    let facts = extract_route_ref_facts(non_helper_expression, "non-helper-expression.ts");
    assert!(facts.route_helpers.is_empty());
}

#[test]
fn summarizes_nested_route_helpers_with_suffixes() {
    let source = r#"
function getTopicTypeSlug(topicType: string): string {
  return topicType;
}
type Topic = { topic_type: string; id: string; slug?: string | null };
export function createTopicPathname(topic: Topic, suffix = ''): string {
  const idOrSlug = topic.slug ?? topic.id;
  return `/${getTopicTypeSlug(topic.topic_type)}/${idOrSlug}${suffix}`;
}
export function topicTagsHref(topic: Topic, tagType: string): string {
  return createTopicPathname(topic, `/tags/${tagType}`);
}
export function topicHref(topic: Topic, tab?: string): string {
  return createTopicPathname(topic, tab ? `/${tab}` : '');
}
"#;
    let facts = extract_route_ref_facts(source, "entity-href.ts");
    let helper = |name: &str| {
        facts
            .route_helpers
            .iter()
            .find(|helper| helper.name == name)
            .map(|helper| helper.patterns.clone())
            .unwrap_or_default()
    };
    assert_eq!(helper("createTopicPathname"), vec!["/*/*"]);
    assert_eq!(helper("topicTagsHref"), vec!["/*/*/tags/*"]);
    assert_eq!(helper("topicHref"), vec!["/*/*", "/*/*/*"]);
}

#[test]
fn summarizes_route_helper_edge_expression_shapes() {
    let source = r#"
export function logicalHref(value: string | null): string {
  return value || `/logical/${value}`;
}
export function assertedHref(value: string): string {
  return (`/asserted/${value}` as string);
}
export function angleAssertedHref(value: string): string {
  return <string>`/angle/${value}`;
}
function objectHref({ id }: { id: string }): string {
  return `/object/${id}`;
}
export function wrappedObjectHref(entity: { id: string }): string {
  return objectHref(entity);
}
export function missingReturnHref(): string {
  return;
}
export function cappedHref(
  a = flag ? '/a' : '/b',
  b = flag ? '/c' : '/d',
  c = flag ? '/e' : '/f',
  d = flag ? '/g' : '/h',
  e = flag ? '/i' : '/j',
): string {
  return a + b + c + d + e;
}
let noInit;
"#;
    let facts = extract_route_ref_facts(source, "edge-shapes.ts");
    let helper = |name: &str| {
        facts
            .route_helpers
            .iter()
            .find(|helper| helper.name == name)
            .map(|helper| helper.patterns.clone())
            .unwrap_or_default()
    };
    assert_eq!(helper("logicalHref"), vec!["/logical/*"]);
    assert_eq!(helper("assertedHref"), vec!["/asserted/*"]);
    assert_eq!(helper("angleAssertedHref"), vec!["/angle/*"]);
    assert_eq!(helper("wrappedObjectHref"), vec!["/object/*"]);
    assert!(helper("missingReturnHref").is_empty());
    let capped_patterns = helper("cappedHref");
    assert_eq!(capped_patterns.len(), 16);
    assert!(capped_patterns.contains(&"/a/c/e/g/i".to_string()));
    assert!(capped_patterns.contains(&"/a/d/f/h/j".to_string()));
    assert!(!capped_patterns.contains(&"/z/y/x/w/v".to_string()));
}

#[test]
fn summarizes_deep_route_helper_calls_as_wildcards() {
    let source = r#"
function passthrough(value: string): string {
  return value;
}
export function deepHref(value: string): string {
  return `/deep/${passthrough(passthrough(passthrough(passthrough(passthrough(value)))))}`;
}
"#;
    let facts = extract_route_ref_facts(source, "deep-href.ts");
    let helper = facts
        .route_helpers
        .iter()
        .find(|helper| helper.name == "deepHref")
        .expect("deep helper should be summarized");
    assert_eq!(helper.patterns, vec!["/deep/*"]);
}

#[test]
fn records_helper_calls_only_in_route_contexts() {
    let source = r#"
import { redirect } from 'next/navigation';
import { entityHref } from './entity-href';
import { type Entity } from './entity-href';

const loose = entityHref(entity);
const link = <Link href={entityHref(entity)} />;
const router = useRouter();
router.push(entityHref(entity));
redirect(entityHref(entity));
fetch(entityHref(entity));
router?.push(entityHref(entity));
router.push?.(entityHref(entity));
redirect?.(entityHref(entity));
globalThis?.fetch(entityHref(entity));
router?.[method](entityHref(entity));
router.push(entityHref?.(entity));
const optionalMember = links?.entityHref;
"#;
    let facts = extract_route_ref_facts(source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec![
            "entityHref",
            "entityHref",
            "entityHref",
            "entityHref",
            "entityHref",
            "entityHref",
            "entityHref",
            "entityHref",
            "entityHref"
        ]
    );
}

#[test]
fn sorts_route_helper_imports_and_refs_deterministically() {
    let source = r#"
import { betaHref } from './b';
import { alphaHref } from './a';
import { entityHref } from './entity-href';

const left = <><Link href={betaHref(entity)} /><Link href={alphaHref(entity)} /></>;
const right = <Link href={entityHref(entity)} />;
"#;
    let facts = extract_route_ref_facts(source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_imports
            .iter()
            .map(|import| {
                (
                    import.local.as_str(),
                    import.imported.as_str(),
                    import.source.as_str(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            ("alphaHref", "alphaHref", "./a"),
            ("betaHref", "betaHref", "./b"),
            ("entityHref", "entityHref", "./entity-href"),
        ]
    );
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| (route_ref.line, route_ref.callee.as_str()))
            .collect::<Vec<_>>(),
        vec![(6, "alphaHref"), (6, "betaHref"), (7, "entityHref")]
    );
}

#[test]
fn records_route_helper_calls_inside_url_wrappers() {
    let source = r#"
import { entityHref } from './entity-href';

const hashLink = <Link href={entityHref(entity) + '#reviews'} />;
const queryLink = <Link href={`${entityHref(entity)}?tab=details`} />;
const prefixLink = <Link href={'/prefix' + entityHref(entity)} />;
const optionalLink = <Link href={entityHref?.(entity)} />;
const loose = entityHref(entity) + '#ignored';
"#;
    let facts = extract_route_ref_facts(source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec!["entityHref", "entityHref", "entityHref", "entityHref"]
    );
}

#[test]
fn records_route_helper_calls_inside_type_wrappers() {
    let source = r#"
import { entityHref } from './entity-href';

const router = useRouter();
router.push(entityHref(entity)!);
router.replace((entityHref(entity) satisfies string));
router.prefetch(<string>entityHref(entity));
router.push(getLinks().entityHref(entity));
"#;
    let facts = extract_route_ref_facts(source, "component.ts");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec!["entityHref", "entityHref", "entityHref"]
    );
}

#[test]
fn records_namespace_helper_calls_in_route_contexts() {
    let source = r#"
import * as links from './entity-href';
const link = <Link href={links.topicHref(topic)} />;
"#;
    let facts = extract_route_ref_facts(source, "component.tsx");
    assert_eq!(facts.route_helper_refs.len(), 1);
    assert_eq!(facts.route_helper_refs[0].callee, "links.topicHref");
    assert_eq!(facts.route_helper_imports[0].imported, "*");
}

#[test]
fn recognizes_optional_member_expressions_as_route_contexts() {
    let allocator = Allocator::default();
    let source = "const router = useRouter(); const maybePush = router?.push;";
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    let mut bindings = collect_import_bindings(&ret.program.body);
    collect_router_bindings_for_scope(&ret.program.body, &mut bindings);
    let Statement::VariableDeclaration(var_decl) = &ret.program.body[1] else {
        panic!("expected variable declaration");
    };
    let expr = var_decl.declarations[0]
        .init
        .as_ref()
        .expect("expected initializer");

    assert!(callee_is_route_context(expr, &bindings));
}

#[test]
fn recognizes_optional_member_expressions_as_helper_callees() {
    let allocator = Allocator::default();
    let source = "const maybeHref = links?.entityHref;";
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    let Statement::VariableDeclaration(var_decl) = &ret.program.body[0] else {
        panic!("expected variable declaration");
    };
    let expr = var_decl.declarations[0]
        .init
        .as_ref()
        .expect("expected initializer");

    assert_eq!(
        route_helper_callee_name(expr),
        Some("links.entityHref".to_string())
    );
    assert_eq!(
        route_helper_callee_name_from_callee(expr),
        Some("links.entityHref".to_string())
    );
}

#[test]
fn extracts_route_refs_from_existing_program() {
    let allocator = Allocator::default();
    let source = "fetch('/program-route');";
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    let refs = extract_route_refs_from_program(&ret.program, source, "program.ts");

    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].pattern, "/program-route");
}
