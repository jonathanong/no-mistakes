import { primary } from '@vendor/valkey';
function replacement() {}
// The binding default runs once per iterated property, after the first write.
for (const { value = ((client = primary) => {
  const get = client.get;
  primary.get = replacement;
  get();
  return 0;
})() } in { first: 1, second: 2 }) {}
