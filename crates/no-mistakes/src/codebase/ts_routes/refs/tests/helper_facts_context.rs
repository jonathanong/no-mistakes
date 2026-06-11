use super::*;

#[test]
fn records_all_helper_calls_inside_route_context_expressions() {
    let source = r#"
import { aHref, bHref } from './entity-href';

const concatLink = <Link href={aHref(a) + bHref(b)} />;
const templateLink = <Link href={`${aHref(a)}${bHref(b)}`} />;
const link = <Link href={flag ? aHref(a) : bHref(b)} />;
const objectLink = <Link href={{ pathname: flag ? aHref(a) : bHref(b) }} />;
const router = useRouter();
router.push(flag ? aHref(a) : bHref(b));
router.replace(aHref(a) || bHref(b));
"#;
    let facts = extract_route_ref_facts(source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec![
            "aHref", "bHref", "aHref", "bHref", "aHref", "bHref", "aHref", "bHref", "aHref",
            "bHref", "aHref", "bHref",
        ]
    );
}

#[test]
fn ignores_shadowed_helper_calls_in_route_contexts() {
    let source = r#"
import { entityHref } from './entity-href';
import * as links from './links';

function Row({ entityHref, links }) {
  return (
    <>
      <Link href={entityHref(row)} />
      <Link href={links.entityHref(row)} />
    </>
  );
}

const link = <Link href={entityHref(entity)} />;
const namespaceLink = <Link href={links.entityHref(entity)} />;
"#;
    let facts = extract_route_ref_facts(source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec!["entityHref", "links.entityHref"]
    );
}

#[test]
fn records_local_aliases_and_namespace_named_imports_in_route_contexts() {
    let source = r#"
import { entityHref } from './entity-href';
import { links } from './links';

const href = entityHref;
const router = useRouter();
router.push(href(entity));
router.replace(links.entityHref(entity));
try {
  router.push(entityHref(entity));
} catch (entityHref) {
  router.push(entityHref(entity));
}
"#;
    let facts = extract_route_ref_facts(source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec!["entityHref", "links.entityHref", "entityHref"]
    );
}
