import { primary } from '@vendor/valkey';
function replacement() {}
// A for update can run repeatedly and must retain the conservative result.
for (
  let index = 0;
  index < 2;
  ((client = primary) => {
    const get = client.get;
    primary.get = replacement;
    get();
    return index++;
  })()
) {}
