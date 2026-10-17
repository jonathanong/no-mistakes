import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// A known direct rebind inside a computed target still invalidates its new receiver.
[sink[(box = { client: replica }, box.client.get = replacement, 'key')]] = [0];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
