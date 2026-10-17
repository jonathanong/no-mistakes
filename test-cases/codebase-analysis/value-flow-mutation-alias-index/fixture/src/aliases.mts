import { receiver, other } from '@vendor/alias-index';
const first = receiver;
const namespace = { method: receiver.get, second: other.get };
const namespaceExtra = receiver;
function outer() {
  // A module write to first must not invalidate this distinct lexical alias.
  const first = other;
  return first;
}
