import { primary } from '@vendor/valkey';
function replacement() {}
primary.get = replacement;
function f(client = primary) { const copied = client.get; const alias = copied; alias(); }
f();
