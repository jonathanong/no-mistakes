import { primary } from '@vendor/valkey';
function replacement() {}
// Combine a tracked direct write with coarse nested propagation.
primary.set = replacement;
const box = { client: primary };
box.client.get = replacement;
function f(client = primary) { const alias = client; alias.get(); }
f();
