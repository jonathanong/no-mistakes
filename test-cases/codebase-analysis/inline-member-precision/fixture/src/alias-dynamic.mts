import { primary } from '@vendor/valkey';
function replacement() {}
declare const key: string;
primary[key] = replacement;
function f(client = primary) { const alias = client; alias.get(); }
f();
