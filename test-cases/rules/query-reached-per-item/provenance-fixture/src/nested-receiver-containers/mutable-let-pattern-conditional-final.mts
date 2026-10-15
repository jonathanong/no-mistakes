import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
declare const choose: boolean;
// Outside a conditional assignment, both its initial and final roots remain possible.
if (choose) {
  [sink[(box = { client: replica }, 'key')]] = [(box = { client }, 0)];
}
box.client.get = replacement;
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
