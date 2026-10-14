import { primary } from '@vendor/valkey';
function replacement() {}
function f() { const snapshot = primary.nested; const first = snapshot; const alias = first; primary.nested = { get: replacement }; alias.get(); }
f();
