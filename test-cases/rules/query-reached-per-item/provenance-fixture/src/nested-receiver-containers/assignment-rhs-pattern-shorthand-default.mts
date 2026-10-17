import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
({ box = { client: replica } } = { box: undefined });
box.client.get = replacement;
function consume(value, other) { value.get('first'); other.get('second'); }
consume(primary, replica);
