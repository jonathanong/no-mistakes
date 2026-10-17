import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
box.client = { get: replacement };
// Eager RHS rebinding to an accessor must not borrow the old root's data proof.
[sink[(box.client.get = replacement, 'key')]] = [(box = { get client() { client.get = replacement; return { get: replacement }; } }, 0)];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
