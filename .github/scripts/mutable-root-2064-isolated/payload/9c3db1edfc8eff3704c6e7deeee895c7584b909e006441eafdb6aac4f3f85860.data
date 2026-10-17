import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
// The LHS receiver is acquired before RHS root rebinding.
box.client.get = (box = { client: replica }, replacement);
function consume(value) { value.get('key'); }
consume(client);
