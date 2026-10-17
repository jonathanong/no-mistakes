import { primary } from '@vendor/valkey';
function replacement() {}
// Setup inherits enclosing execution; per-item positions must remain repeatable.
async function run() { for (const value of [((client = primary) => { const get = client.get; primary.get = replacement; get(); return 1; })()]) {} }
run(); run();
