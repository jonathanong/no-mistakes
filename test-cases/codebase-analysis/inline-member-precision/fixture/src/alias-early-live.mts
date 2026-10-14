import { primary } from '@vendor/valkey';
function replacement() {}
function f(client = primary) { primary.get = replacement; const alias = client; alias.get(); }
f();
