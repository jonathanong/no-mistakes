import { primary } from '@vendor/valkey';
function replacement() {}
primary.get = replacement;
function f(client = primary) { const alias = client; alias.get(); }
f();
