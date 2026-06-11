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
fn records_helper_calls_only_in_route_contexts() {
    let source = r#"
import { redirect } from 'next/navigation';
import { entityHref } from './entity-href';

const loose = entityHref(entity);
const link = <Link href={entityHref(entity)} />;
const router = useRouter();
router.push(entityHref(entity));
redirect(entityHref(entity));
fetch(entityHref(entity));
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
fn records_route_helper_calls_inside_url_wrappers() {
    let source = r#"
import { entityHref } from './entity-href';

const hashLink = <Link href={entityHref(entity) + '#reviews'} />;
const queryLink = <Link href={`${entityHref(entity)}?tab=details`} />;
const loose = entityHref(entity) + '#ignored';
"#;
    let facts = extract_route_ref_facts(source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec!["entityHref", "entityHref"]
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
