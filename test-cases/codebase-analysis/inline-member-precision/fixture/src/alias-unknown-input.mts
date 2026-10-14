import { primary } from '@vendor/valkey';
declare const unknownClient: unknown;
function f(client = primary) { const alias = client; alias.get(); }
f();
f(unknownClient);
