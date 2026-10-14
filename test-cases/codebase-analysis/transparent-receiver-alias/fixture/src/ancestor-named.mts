import { primary } from '@vendor/valkey';
function replacement() {}
primary.nested = {get: replacement};
function f(client = primary) { const alias = client; alias.nested.get(); }
f();
