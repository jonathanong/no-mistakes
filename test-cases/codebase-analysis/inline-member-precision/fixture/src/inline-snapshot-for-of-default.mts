import { primary } from '@vendor/valkey';
function replacement() {}
// The binding default runs once per iterated value, after the first write.
for (const { value = ((client = primary) => {
  const get = client.get;
  primary.get = replacement;
  get();
  return 0;
})() } of [1, 2]) {}
