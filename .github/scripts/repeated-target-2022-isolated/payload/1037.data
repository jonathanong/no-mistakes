import { primary } from '@vendor/valkey';
function replacement() {}
function f(client) { const alias = client; const get = alias.nested.get; primary.nested.get = replacement; get(); }
f(primary);
