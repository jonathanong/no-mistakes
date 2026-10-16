import { primary } from '@vendor/valkey';
const eager = require('./ordinary-eager.cjs');
let box = { client: primary };
function replacement() {}
box.client.get = replacement;
box.client ??= { get: replacement };
const accessor = {
  get value() { return primary.get; },
  set value(next) { primary.get = next; },
};
accessor.value ||= primary.get;
// The default and callable must still be visited when receiver facts are cold.
({ value: box.client = primary } = accessor);
function deferred(callback = primary.get) { return callback; }
class Receiver {
  static eager = require('./ordinary-static.cjs');
  field = (() => { box.client = primary; return require('./ordinary-deferred.cjs'); })();
  get value() { return primary.get; }
  set value(next) { primary.get = next; }
}
if (condition) require('./ordinary-conditional.cjs');
new Receiver();
deferred();
eager;
