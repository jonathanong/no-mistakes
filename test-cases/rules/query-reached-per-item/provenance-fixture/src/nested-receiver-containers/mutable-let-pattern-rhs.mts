import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// RHS runs before any targets; target elements execute from left to right.
[box] = [(box.client.get = replacement, { client: replica })];
function consume(value) { value.get('key'); }
consume(client);
