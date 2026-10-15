import { primary } from '@vendor/valkey';
function replacement() {}
// A classic for initializer runs once before the first test.
for (let value = ((client = primary) => {
  const get = client.get;
  primary.get = replacement;
  get();
  return 0;
})(); false;) {}
