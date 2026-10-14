import { primary } from '@vendor/valkey';
function replacement() {}
function f(client = primary) { const alias = client; const get = alias.get; primary.get = replacement; get.bind(null)(); }
f();
