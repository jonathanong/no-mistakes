import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
declare const choose: boolean;
// If the assignment executes, its root reset dominates the later target.
if (choose) {
  [box, sink[(box.client.get = replacement, 'key')]] = [{ client: replica }, 0];
}
function consume(value) { value.get('key'); }
consume(client);
