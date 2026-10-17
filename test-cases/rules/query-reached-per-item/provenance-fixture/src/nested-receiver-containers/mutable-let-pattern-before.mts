import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// Preserve the earliest root-write phase and earlier receiver acquisitions.
[sink[(box.client.get = replacement, 'key')], box] = [0, { client: replica }];
function consume(value) { value.get('key'); }
consume(client);
