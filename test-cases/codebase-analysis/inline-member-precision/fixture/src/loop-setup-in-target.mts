import { primary } from '@vendor/valkey';
function replacement() {}
// Setup inherits enclosing execution; per-item positions must remain repeatable.
for (primary[((client = primary) => { const get = client.get; primary.get = replacement; get(); return 1; })()] in { a: 1, b: 2 }) {}
