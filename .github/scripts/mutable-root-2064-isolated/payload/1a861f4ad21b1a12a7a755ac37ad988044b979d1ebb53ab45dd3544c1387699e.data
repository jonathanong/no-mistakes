import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// The ordinary possible-origin set must retain the final phase-selected root.
[sink[(box = { client: replica }, 'key')]] = [(box = { client }, 0)];
function mutate() { box.client.get = replacement; }
mutate();
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
