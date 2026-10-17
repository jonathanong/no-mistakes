import { primary } from '@vendor/valkey';
function replacement() {}
// Each body execution can acquire a replacement.
for (const key in { a: 1, b: 2 }) { ((client = primary) => { const get = client.get; primary.get = replacement; get(); return 1; })(); }
