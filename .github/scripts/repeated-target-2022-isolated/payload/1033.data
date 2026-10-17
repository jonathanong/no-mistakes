import { primary } from '@vendor/valkey';
function replacement() {}
function f(client) { primary.get = replacement; const alias = client; alias.get(); }
f(primary);
