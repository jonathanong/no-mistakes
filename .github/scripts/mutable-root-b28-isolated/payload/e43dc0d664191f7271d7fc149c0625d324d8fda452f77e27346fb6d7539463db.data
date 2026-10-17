import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// Eager static evaluation precedes the LHS receiver acquisition.
[sink[(box.client.get = replacement, 'key')]] = [class { static { box = { client: replica }; } }];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
