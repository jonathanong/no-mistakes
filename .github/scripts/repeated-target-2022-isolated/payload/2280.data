import { foreignSnapshot } from '@vendor/valkey';
import { acquireCopied, acquireLive, acquireAfter, mutateForeign } from './foreign-repeated-helper.mjs';
function replacement() {}
function forwardCopied() { return acquireCopied(); }
for (const key of ['a', 'b']) {
  (function foreignCopied(fn) { fn(key); })(forwardCopied());
  foreignSnapshot.get = replacement;
}
function forwardLive() { return acquireLive(); }
for (const key of ['a', 'b']) {
  (function foreignRepeated(fn) { fn(key); })(forwardLive());
  mutateForeign();
}
function forwardAfter() { return acquireAfter(); }
for (const key of ['a', 'b']) {
  (function foreignAfterWrite(fn) { fn(key); })(forwardAfter());
}
