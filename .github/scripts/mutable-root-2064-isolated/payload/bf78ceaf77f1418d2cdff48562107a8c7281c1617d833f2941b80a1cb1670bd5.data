import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
box.client = { get: replacement };
// The first LHS still reads the detached member; a later LHS root write cannot change that acquisition.
[sink[(box.client.get = replacement, 'key')], box] = [0, { client: replica }];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(client, replica);
