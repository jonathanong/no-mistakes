import { primary } from '@vendor/valkey';
function replacement() {}
// Each body execution can acquire a replacement.
for (const value of [1, 2]) { ((client = primary) => { const get = client.get; primary.get = replacement; get(); return 1; })(); }
