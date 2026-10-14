import { primary } from '@vendor/valkey';
function replacement() {}
const copied = { get: primary.get };
primary.get = replacement;
function f(client = copied) { const alias = client; alias.get(); }
f();
