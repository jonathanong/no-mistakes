import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
function mutate() {
  // Phase origins exist before this callable has an ordinary root snapshot.
  [sink[(box.client.get = replacement, 'key')]] = [(box = { client: replica }, 0)];
}
mutate();
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
