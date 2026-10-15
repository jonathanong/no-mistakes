import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
box.client = { get: replacement };
// This earlier LHS rebind changes the receiver before the later acquisition.
[sink[(box = { client: replica }, 'rebind')], sink[(box.client.get = replacement, 'key')]] = [0, 0];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
