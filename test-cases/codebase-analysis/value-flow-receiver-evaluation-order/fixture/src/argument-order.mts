import {
  primary, replica, later, copied, objectCopy, inline, inlineCopy,
  returned, bound, boundCopy, defaults, defaultMethod, rhs, spread,
} from '@vendor/valkey';
function replacement() {}

function before(client) { client.get('key'); }
before(primary);
function during(client) { client.get('key'); client.set('key', 'value'); }
during(primary, primary.get = replacement);
function otherReceiver(client) { client.get('key'); }
otherReceiver(replica, primary.get = replacement);
function after(client) { client.get('key'); }
after(later);
later.get = replacement;

// Earlier arguments acquire methods immediately, while receivers stay live.
function copiedMethod(get) { get('key'); }
copiedMethod(copied.get, copied.get = replacement);
function copiedObject(options) { options.get('key'); }
copiedObject({ get: objectCopy.get }, objectCopy.get = replacement);
(function inlineDuring(client) { client.get('key'); })(inline, inline.get = replacement);
(function inlineCopied(get) { get('key'); })(inlineCopy.get, inlineCopy.get = replacement);
function factory() { return function returnedDuring(client) { client.get('key'); }; }
factory()(returned, returned.get = replacement);
function boundDuring(client) { client.get('key'); }
boundDuring.bind(null, bound)(bound.get = replacement);
function boundCopied(get) { get('key'); }
boundCopied.bind(null, boundCopy.get)(boundCopy.get = replacement);

function defaultReceiver(client = defaults) { client.get('key'); }
defaultReceiver(undefined, defaults.get = replacement);
(function inlineDefault(get = defaultMethod.get) { get('key'); })(undefined, defaultMethod.get = replacement);
// The helper executes before the enclosing assignment completes, even when
// its exclusive source end is the same as the enclosing assignment's end.
function assignmentRhs(client) { client.get('key'); return replacement; }
rhs.get = assignmentRhs(rhs);
function afterRhs(client) { client.get('key'); }
afterRhs(rhs);
function spreadUnknown(client) { client.get('key'); }
declare const opaque: unknown[];
spreadUnknown(...opaque, spread.get = replacement);
