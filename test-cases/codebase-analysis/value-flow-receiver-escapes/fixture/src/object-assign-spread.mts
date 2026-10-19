import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
const source = { client: replica };
function write() { box.client.get = replacement; }
// A spread array is still a source object, not an unknown prefix.
Object.assign(box, ...[source]);
write();
replica.get('mutated');
primary.get('old');
unrelated.get('unrelated');
