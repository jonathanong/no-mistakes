import { primary as client, replica } from '@vendor/valkey';
let box = { client };
let other = { client: replica };
const sink = {};
function replacement() {}
// Root writes in the RHS remain visible when reference proofs resume for the LHS.
[sink[(box.client.get, box = { client }, other = { client }, box = { client }, 'key')]] = [
  (box = { client }, other = { client },
    class { static field = (box.client.get = replacement, 0); field = other.client.get; }),
];
client.get('key');
replica.get('key');
