import * as UI from "../components/button";

// Namespace import + member callsite resolves through the "*" export. Also
// exercises a namespaced JSX attribute (data:role) and an unresolved uppercase
// component (<Unresolved />), which must not produce a callsite.
export function NamespacedUser() {
  return (
    <>
      <UI.Button variant="secondary" onClick={() => {}} data:role="primary" />
      <Unresolved />
    </>
  );
}
