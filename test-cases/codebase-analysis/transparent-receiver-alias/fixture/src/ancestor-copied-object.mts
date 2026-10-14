import { primary } from '@vendor/valkey';
function replacement() {}
function f() { const snapshot = primary.nested; const alias = snapshot; primary.nested = { get: replacement }; alias.get(); }
f();
