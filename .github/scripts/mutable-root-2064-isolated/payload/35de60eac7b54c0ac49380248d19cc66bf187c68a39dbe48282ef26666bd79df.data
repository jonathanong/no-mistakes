import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
box.client = { get: replacement };
// The RHS chooses a new root, so the old member replacement cannot hide its write.
[sink[(box.client.get = replacement, 'key')]] = [(box = { client: replica }, 0)];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
