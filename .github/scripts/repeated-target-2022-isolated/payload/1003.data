import { primary } from '@vendor/valkey';
function replacement() {}
function f(client = primary) { const alias = client; return () => alias.get(); }
const run = f();
primary.get = replacement;
run();
