import { primary } from '@vendor/valkey';
function replacement() {}
// The for-in iterable expression is evaluated once before enumeration.
for (const key in ((client = primary) => {
  const get = client.get;
  primary.get = replacement;
  get();
  return { key: true };
})()) {}
