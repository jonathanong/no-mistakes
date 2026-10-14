import { primary } from '@vendor/valkey';
function replacement() {}
function run() { const snapshot = primary.nested; primary.nested = {get: replacement}; ((client) => { const alias = client; alias.get(); })(snapshot); }
run();
