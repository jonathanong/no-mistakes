import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
const sink = {};
let outer = { client }; // Keep the enclosing mutable-root catalog active.
// A class-local root is initialized after the enclosing catalog starts.
[sink.key] = [class { static {
  let box = { client };
  [sink[(box.client.get = replacement, 'key')]] = [(box = { client: replica }, 0)];
} }];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
