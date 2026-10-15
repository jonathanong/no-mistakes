import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const source = { selected: { client: replica } };
({ selected: box } = source);
box.client.get = replacement;
function consume(value, other) { value.get('first'); other.get('second'); }
consume(primary, replica);
