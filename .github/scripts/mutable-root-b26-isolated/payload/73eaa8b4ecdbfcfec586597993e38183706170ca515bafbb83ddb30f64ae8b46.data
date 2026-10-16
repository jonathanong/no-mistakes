import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// A shorthand lexical target resets the root before the next property target.
({ box, other: sink[(box.client.get = replacement, 'key')] } = { box: { client: replica }, other: 0 });
function consume(value) { value.get('key'); }
consume(client);
