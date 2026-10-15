import { primary, replica } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
let box = { client: primary };
// Both runtime branches select the fallback; opaque projection must retain it.
({ selected: box = { client: replica } } = flag ? {} : {});
box.client.get = replacement;
function consume(value, other) { value.get('first'); other.get('second'); }
consume(primary, replica);
