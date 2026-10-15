import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const sink = {};
// The first target installs the RHS identity before the later computed target reads it.
[box, sink[(box.client.get = replacement, 'key')]] = [{ client: replica }, 0];
function consume(value, changed) { value.get('key'); changed.get('after'); }
consume(primary, replica);
