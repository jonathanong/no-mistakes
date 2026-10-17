import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// Constructing the class never executes its instance initializer.
[sink[(box.client.get = replacement, 'key')]] = [class { field = (box = { client: replica }); }];
function consume(value, changed) { value.get('key'); changed.get('key'); }
consume(client, replica);
