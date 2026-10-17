import { primary } from '@vendor/valkey';
function replacement() {}
function acquire(fn = primary.get) { return fn; }
function forward() { return acquire(); }
for (const key of ['a', 'b']) {
  ((fn) => fn(key))(forward());
  primary.get = replacement;
}
