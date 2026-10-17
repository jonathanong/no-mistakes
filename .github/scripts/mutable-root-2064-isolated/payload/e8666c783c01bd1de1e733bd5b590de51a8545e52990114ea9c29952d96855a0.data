import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// The RHS rebinds before acquiring the receiver in the computed LHS.
[sink[(box.client.get = replacement, 'key')]] = [(box = { client: replica }, 0)];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
