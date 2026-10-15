import { primary } from '@vendor/valkey';
function replacement() {}
const saved = primary.get;
function acquire(fn = saved) { return fn; }
function forward() { return acquire(); }
for (const key of ['a', 'b']) {
  ((fn) => fn(key))(forward());
  primary.get = replacement;
}
