import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
function f(client) { const alias = client; alias.get(); }
f(primary);
