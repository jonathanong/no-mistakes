import { primary } from '@vendor/valkey';
function replacement() {}
function f(client) { const alias = client; const nested = alias.nested; primary.nested.set = replacement; nested.get(); }
f(primary);
