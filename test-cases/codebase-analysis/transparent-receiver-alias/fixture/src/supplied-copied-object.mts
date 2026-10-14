import { primary } from '@vendor/valkey';
function replacement() {}
function f(client) { const alias = client; alias.get(); }
function run() { const snapshot = primary.nested; primary.nested = {get: replacement}; f(snapshot); }
run();
