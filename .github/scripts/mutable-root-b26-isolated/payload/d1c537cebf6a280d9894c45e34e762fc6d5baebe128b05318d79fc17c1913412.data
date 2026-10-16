import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// The RHS source acquires the old root before its own replacement completes.
[sink[(box.client.get = replacement, 'key')]] = [(box = { client: box.client }, 0)];
function consume(value) { value.get('key'); }
consume(client);
