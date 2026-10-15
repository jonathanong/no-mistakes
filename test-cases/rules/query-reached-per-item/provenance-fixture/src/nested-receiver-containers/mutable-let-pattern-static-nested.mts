import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// The physically earlier outer LHS rebind executes after this RHS root read.
[sink[(box = { client: replica }, 'key')]] = [class { static {
  [sink[(box.client.get = replacement, 'nested')]] = [0];
} }];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
