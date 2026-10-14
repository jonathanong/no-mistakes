import { primary } from '@vendor/valkey';
function f(client = primary) { const first = client; const alias = first; alias.get(); }
f();
