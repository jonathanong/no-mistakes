import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// Preserve the earliest root-write phase and earlier receiver acquisitions.
[(box as any), sink[(box.client.get = replacement, 'key')]] = [{ client: replica }, 0];
function consume(value) { value.get('key'); }
consume(client);
