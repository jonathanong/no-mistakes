import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
declare const choose: boolean;
// A conditional RHS rebind keeps both possible receiver roots.
[sink[(box.client.get = replacement, 'key')]] = [(choose && (box = { client: replica }), 0)];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
