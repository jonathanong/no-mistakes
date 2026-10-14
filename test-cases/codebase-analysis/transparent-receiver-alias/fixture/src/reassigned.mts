import { primary } from '@vendor/valkey';
function replacement() {}
function f(client = primary) { let alias = client; alias = { get: replacement }; alias.get(); }
f();
