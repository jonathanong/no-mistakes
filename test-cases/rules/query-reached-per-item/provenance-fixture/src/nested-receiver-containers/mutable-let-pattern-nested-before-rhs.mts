import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// The outer RHS reads the old root before the physically earlier nested LHS reset.
[box, sink[([box] = [(box.client.get = replacement, { client: replica })], 'k')]] = [(box.client.get = replacement, { client: replica }), 0];
function consume(value) { value.get('key'); }
consume(client);
