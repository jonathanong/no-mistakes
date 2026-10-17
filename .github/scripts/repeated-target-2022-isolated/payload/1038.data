import { primary } from '@vendor/valkey';
function replacement() {}
function f(client) { const alias = client; const get = alias.get; primary.get = replacement; get(); }
f(primary);
