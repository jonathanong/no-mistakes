import { primary } from '@vendor/valkey';
function replacement() {}
// An inner iterable expression repeats with its enclosing loop.
for (let outer = 0; outer < 2; outer++) {
  for (const value of ((client = primary) => {
    const get = client.get;
    primary.get = replacement;
    get();
    return [0];
  })()) {}
}
