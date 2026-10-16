import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// A logical RHS root replacement contributes its receiver as a possible origin.
[sink[(box.client.get = replacement, 'key')]] = [(box &&= { client: replica }, 0)];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
