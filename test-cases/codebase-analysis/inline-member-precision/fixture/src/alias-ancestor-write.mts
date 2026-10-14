import { primary } from '@vendor/valkey';
function replacement() {}
// A whole-object alias keeps descendant container replacement live.
function f(client = primary) { const alias = client; primary.nested = { get: replacement }; alias.nested.get(); }
f();
