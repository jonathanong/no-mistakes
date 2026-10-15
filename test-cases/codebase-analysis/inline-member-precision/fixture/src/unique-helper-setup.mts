import { primary } from '@vendor/valkey';
function replacement() {}
function acquire(fn = primary.get) { return fn; }
function forward() { return acquire(); }
for (((fn) => fn('key'))(forward()); false;) {
  primary.get = replacement;
}
