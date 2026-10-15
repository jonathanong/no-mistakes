import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
let first, second;
({ first: first, ['second']: second, ...box } = { first: 0, second: 0, client: replica });
box.client.get = replacement;
function consume(value, other) { value.get('first'); other.get('second'); }
consume(primary, replica);
