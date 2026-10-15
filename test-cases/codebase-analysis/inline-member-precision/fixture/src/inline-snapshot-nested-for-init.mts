import { primary } from '@vendor/valkey';
function replacement() {}
// An inner setup expression repeats when its enclosing loop iterates.
for (let outer = 0; outer < 2; outer++) {
  for (let value = ((client = primary) => {
    const get = client.get;
    primary.get = replacement;
    get();
    return 0;
  })(); false;) {}
}
