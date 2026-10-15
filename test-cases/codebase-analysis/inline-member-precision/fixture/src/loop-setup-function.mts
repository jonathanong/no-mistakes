import { primary } from '@vendor/valkey';
function replacement() {}
// Setup inherits enclosing execution; per-item positions must remain repeatable.
function run() { for (((client = primary) => { const get = client.get; primary.get = replacement; get(); return 1; })(); false;) {} }
run(); run();
