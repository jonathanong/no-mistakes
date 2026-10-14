import { merged, nested, dynamic, arrayRest, objectRest, shorthand, ordinary, updated, deferred, owned, outer } from '@vendor/valkey';
function replacement() {}
function read(value) { return value; }
let unused, key = 'get';
const alias = merged;
// Histories from both aliases must coalesce before catalog lookup.
({ first: merged.get, second: alias.get, third: alias[key], later: unused = read(merged) } = {});
({ first: nested.client.get, later: unused = read(nested) } = {});
({ first: dynamic[key].get, later: unused = read(dynamic) } = {});
// Rest-only receiver targets must retain root-RHS-before-target ordering.
[unused, ...arrayRest.get] = read(arrayRest);
read(arrayRest);
({ field: unused, ...objectRest.get } = read(objectRest));
read(objectRest);
({ first: shorthand.get, unused = read(shorthand) } = {});
// Ordinary assignment/update in a nested RHS completes before its LHS default.
({ first: outer.get, later: unused = ({ field: unused = read(ordinary) } = (ordinary.get = replacement, {})) } = {});
({ first: outer.set, later: unused = ({ field: unused = read(updated) } = (updated[key].get++, {})) } = {});
// A function expression is deferred syntax, not an outer evaluation segment.
({ first: deferred.get, later: unused = function saved() { read(owned); ({ first: owned.get, later: unused = read(owned) } = {}); return read(deferred); }, after: unused = read(deferred) } = {});
