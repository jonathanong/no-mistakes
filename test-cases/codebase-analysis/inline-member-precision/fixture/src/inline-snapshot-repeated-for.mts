import { primary } from '@vendor/valkey';
function replacement() {}
// The next iteration can acquire the replacement, so a single original target is unsafe.
for (let index = 0; index < 2; index++) {
  ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
}
