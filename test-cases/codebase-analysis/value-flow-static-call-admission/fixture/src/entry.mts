import { read, write } from '@vendor/static-admission';
import { imported } from './helper.mts';
const receiver = { get: read };
function direct(value = 'default') { return value; }
function enclosing() {
  function direct(value = 'nested') { return value; }
  direct();
}
// A named initializer gives one physical call multiple caller projections.
const savedResult = direct('saved');
direct('first');
direct('second');
enclosing();
{
  // Same display name, different lexical scope and parser callable identity.
  function direct(value = 'block') { return value; }
  direct();
}
function supplied(direct = receiver.get) { direct('parameter'); }
const copied = receiver.get;
const alias = direct;
const bound = direct.bind(null);
const defaulted = maybe ?? receiver.get;
const selected = condition ? receiver.get : direct;
let reassigned = direct;
reassigned = imported;
class Getter {
  get method() { return receiver.get; }
}
const getter = new Getter();
for (const item of ['a', 'b']) {
  direct(item);
  imported(item);
  supplied();
  copied(item);
  alias(item);
  bound(item);
  defaulted(item);
  selected(item);
  reassigned(item);
  getter.method(item);
  receiver['get'](item);
  receiver.get = write;
  receiver.get(item);
}
// A for-of target invalidates a declaration; it must not become a static proof.
function iterationTarget() { return 'before'; }
for (iterationTarget of [direct, imported]) { iterationTarget(); }
