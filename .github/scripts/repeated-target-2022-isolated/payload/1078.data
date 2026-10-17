import { primary } from '@vendor/valkey';
function replacement() {}
let index = 0;
// A later while iteration can acquire the replacement.
while (index++ < 2) {
  ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
}
