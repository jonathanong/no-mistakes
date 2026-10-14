import { primary } from '@vendor/valkey';
function replacement() {}
function f(client = primary) { const get = client.get; primary.get = replacement; get(); }
f();
