import { primary } from '@vendor/valkey';
function replacement() {}
function f(client = primary) { const alias = client; const nested = alias.nested; primary.nested.get = replacement; nested.get.call(null); }
f();
