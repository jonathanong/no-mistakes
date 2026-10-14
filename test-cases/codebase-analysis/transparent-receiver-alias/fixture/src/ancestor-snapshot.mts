import { primary } from '@vendor/valkey';
function replacement() {}
const copied = { get: primary.nested.get };
primary.nested = {get: replacement};
function f(client = copied) { const alias = client; alias.get(); }
f();
