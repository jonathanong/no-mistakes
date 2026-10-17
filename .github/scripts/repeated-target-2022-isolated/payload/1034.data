import { primary } from '@vendor/valkey';
function replacement() {}
function f(client) { const alias = client; primary.get = replacement; alias.get(); }
f(primary);
