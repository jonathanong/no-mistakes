import { primary, replica } from '@vendor/valkey';
function replacement() {}
// Named rest remains conservative without a literal selector projection.
const source = { client: replica };
let box = { client: primary };
({...box} = source);
box.client.get = replacement;
consume(box.client);
function consume(value) { value.get('changed'); }
