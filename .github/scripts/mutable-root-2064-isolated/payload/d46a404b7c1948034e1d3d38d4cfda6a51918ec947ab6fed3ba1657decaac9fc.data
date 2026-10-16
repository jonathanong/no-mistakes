import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
const carrier = { client };
carrier.client = { get: replacement };
let box = { carrier };
const sink = {};
// The stable immutable carrier's detached member stays detached through a mutable wrapper.
[sink[(box.carrier.client.get = replacement, 'key')]] = [0];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
