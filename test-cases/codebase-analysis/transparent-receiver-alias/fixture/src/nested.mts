import { primary } from '@vendor/valkey';
function replacement() {}
primary.nested.set = replacement;
function f(client = primary) { const alias = client; alias.nested.get(); }
f();
