import { primary } from '@vendor/valkey';
function replacement() {}
let key = 'get';
primary[key] = replacement;
function f(client = primary) { const alias = client; alias.get(); }
f();
