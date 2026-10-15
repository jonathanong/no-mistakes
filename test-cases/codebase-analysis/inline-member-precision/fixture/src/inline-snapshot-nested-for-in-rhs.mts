import { primary } from '@vendor/valkey';
function replacement() {}
// An inner for-in iterable repeats with its enclosing loop.
for (let outer = 0; outer < 2; outer++) {
  for (const key in ((client = primary) => {
    const get = client.get;
    primary.get = replacement;
    get();
    return { key: true };
  })()) {}
}
