import { primary } from '@vendor/valkey';
function replacement() {}
// A for-in binding default runs once per property, so it remains repeatable.
for (const { value = ((client = primary) => { const get = client.get; primary.get = replacement; get(); return 1; })() } in { first: 1, second: 2 }) {}
