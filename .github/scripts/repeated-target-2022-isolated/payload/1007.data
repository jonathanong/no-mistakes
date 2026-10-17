import { primary } from '@vendor/valkey';
function replacement() {}
primary.nested.set = replacement;
function f(client = primary.nested) { const alias = client; alias.get(); }
f();
