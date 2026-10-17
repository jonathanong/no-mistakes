import { primary, replica, scoped, session, nested, local, standalone, reversed, arrayClient, computed, dynamic, ownDefault, overlap } from '@vendor/valkey';
function replacement() {}
let unused;
function after(client) { client.get('key'); }
// The first target is written before the second target's default executes.
({ first: primary.get, second: unused = after(primary) } = { first: replacement });
function before(client) { client.get('key'); }
({ first: unused = before(replica), second: replica.get } = { second: replacement });
function rhs(client) { client.get('key'); return { first: replacement }; }
function afterRhs(client) { client.get('key'); }
// The RHS runs before either target, despite appearing last in source text.
({ first: scoped.get, second: unused = afterRhs(scoped) } = rhs(scoped));
function untouched(client) { client.info('key'); }
({ first: session.get, second: unused = untouched(session) } = { first: replacement });
const saved = nested.get;
function copied(get) { get('key'); }
({ first: nested.get, second: unused = copied(saved) } = { first: replacement });
function copiedAfter(get) { get('key'); }
({ first: local.get, second: unused = copiedAfter(local.get) } = { first: replacement });
function nestedAfter(client) { client.get('key'); }
function enclosingDefault(client) { client.get('key'); return {}; }
// An enclosing default runs before its inner pattern; inner later defaults run after its first write.
({ outer: { first: standalone.get, second: unused = nestedAfter(standalone) } = enclosingDefault(standalone) } = {});
function outerAfter(client) { client.get('key'); }
({ outer: { first: reversed.get } = {}, after: unused = outerAfter(reversed) } = {});
function arrayAfter(client) { client.get('key'); }
[arrayClient.get, unused = arrayAfter(arrayClient)] = [replacement];
function computedAfter(client) { client.get('key'); return 'second'; }
({ first: computed.get, [computedAfter(computed)]: unused } = { first: replacement });
function dynamicAfter(client) { client.get('key'); }
let key = 'get';
({ first: dynamic[key], second: unused = dynamicAfter(dynamic) } = { first: replacement });
function ownBefore(client) { client.get('key'); return replacement; }
({ first: ownDefault.get = ownBefore(ownDefault) } = {});
function overlappingAfter(client) { client.get('key'); }
// Repeated targets overlap in the shared early-read interval union.
({ first: overlap.get, second: overlap.get, third: unused = overlappingAfter(overlap) } = { first: replacement, second: replacement });
