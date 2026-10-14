import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
function f(client = primary) { const first = client; const alias = (first as any); alias.get(); }
f();
