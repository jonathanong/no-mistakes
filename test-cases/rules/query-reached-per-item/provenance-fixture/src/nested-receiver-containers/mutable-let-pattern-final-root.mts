import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// The RHS is physically later, but the LHS rebind determines the final root.
[sink[(box = { client: replica }, 'key')]] = [(box = { client }, 0)];
box.client.get = replacement;
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
