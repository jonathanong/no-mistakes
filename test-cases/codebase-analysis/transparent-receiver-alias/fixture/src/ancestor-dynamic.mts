import { primary } from '@vendor/valkey';
function replacement() {}
declare const key: string;
primary[key] = {get: replacement};
function f(client = primary) { const alias = client; alias.nested.get(); }
f();
