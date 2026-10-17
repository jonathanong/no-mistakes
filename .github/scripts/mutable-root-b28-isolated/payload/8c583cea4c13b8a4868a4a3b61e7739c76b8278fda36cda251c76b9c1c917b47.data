import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// This block-local binding must never replace the module's root.
[sink[(box.client.get = replacement, 'key')]] = [class { static { let box = {}; box = { client: replica }; } }];
function consume(value, changed) { value.get('key'); changed.get('key'); }
consume(client, replica);
