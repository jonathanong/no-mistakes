import { primary } from '@vendor/valkey';
function replacement() {}
primary.nested.get = replacement;
function f(client = primary.nested) { const alias = client; alias.get(); }
f();
