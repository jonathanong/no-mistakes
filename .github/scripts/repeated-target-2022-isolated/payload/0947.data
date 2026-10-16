import { outer, inner, root, rhsNested, fnReceiver, updated } from '@vendor/valkey';
function replacement() {}
let unused;
function afterInner(client) { client.get(); }
function afterOuter(client) { client.get(); }
function nestedRhs(first, second) { first.get(); second.get(); return { first: replacement }; }
// The nested RHS sees the outer write, but precedes its own target writes.
({ first: outer.get, later: unused = ({ first: inner.get, later: unused = afterInner(inner), outer: unused = afterOuter(outer) } = nestedRhs(inner, outer)) } = { first: replacement });
function source(client) { client.get(); return { first: replacement }; }
function afterNested(client) { client.get(); }
function afterRoot(client) { client.get(); }
function rootRhs(value, client) { client.get(); return { first: replacement }; }
// A nested assignment in the root RHS owns a separate catalog.
({ first: root.get, later: unused = afterRoot(root) } = rootRhs(({ first: rhsNested.get, later: unused = afterNested(rhsNested) } = source(rhsNested)), root));
function afterFunction(client) { client.get(); }
// Deferred callable bodies are excluded from the outer catalog, then indexed in their own context.
({ first: outer.set, later: unused = (() => { ({ first: fnReceiver.get, later: unused = afterFunction(fnReceiver) } = { first: replacement }); })() } = { first: replacement });
function afterUpdate(client) { client.get(); }
// The nested RHS update completes before its textually earlier default read.
({ first: outer.info, later: unused = ({ later: unused = afterUpdate(updated) } = (updated.get++, {})) } = {});
