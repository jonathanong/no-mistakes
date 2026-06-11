use super::*;

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
