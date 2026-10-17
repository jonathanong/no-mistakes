import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// RHS runs before any targets; target elements execute from left to right.
[box, sink[(box.client.get = replacement, 'key')]] = [{ client: replica }, 0];
function consume(value) { value.get('key'); }
consume(client);
