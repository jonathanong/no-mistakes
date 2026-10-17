import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
[box = { client: replica }] = [];
box.client.get = replacement;
function consume(value, other) { value.get('original'); other.get('changed'); }
consume(primary, replica);
