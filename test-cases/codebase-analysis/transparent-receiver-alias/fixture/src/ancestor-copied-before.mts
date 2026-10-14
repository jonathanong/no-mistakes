import { primary } from '@vendor/valkey';
function replacement() {}
function f() { primary.nested = {get: replacement}; const snapshot = primary.nested; const alias = snapshot; alias.get(); }
f();
