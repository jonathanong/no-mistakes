import { primary } from '@vendor/valkey';
function replacement() {}
// The for-of iterable expression is evaluated once before iteration.
for (const value of ((client = primary) => {
  const get = client.get;
  primary.get = replacement;
  get();
  return [0];
})()) {}
