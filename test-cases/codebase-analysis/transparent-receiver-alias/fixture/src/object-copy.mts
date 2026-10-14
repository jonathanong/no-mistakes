import { primary } from '@vendor/valkey';
function replacement() {}
primary.get = replacement;
function f(client = { get: primary.get }) { const alias = client; alias.get(); }
f();
