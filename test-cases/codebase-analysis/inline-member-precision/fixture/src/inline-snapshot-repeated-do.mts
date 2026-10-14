import { primary } from '@vendor/valkey';
function replacement() {}
let index = 0;
// A later do iteration can acquire the replacement.
do {
  ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
} while (++index < 2);
