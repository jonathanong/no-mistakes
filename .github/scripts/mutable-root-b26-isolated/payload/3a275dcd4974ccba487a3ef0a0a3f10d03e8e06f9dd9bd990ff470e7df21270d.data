import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// This static block shares the callable owner but has its own lexical root scope.
[sink.outer] = [class { static {
  let inner = { client: replica };
  [sink[(inner.client.get = replacement, 'key')]] = [(inner = { client }, 0)];
} }];
function consume(value, changed) { value.get('key'); changed.get('key'); }
consume(client, replica);
