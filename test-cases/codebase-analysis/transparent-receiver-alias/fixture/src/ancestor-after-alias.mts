import { primary } from '@vendor/valkey';
function replacement() {}
function f(client = primary) { const alias = client; primary.nested = {get: replacement}; alias.nested.get(); }
f();
