import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const other = { client: primary };
function mutate(value) { value.client.get = replacement; }
box = { client: replica };
mutate(box ?? other);
replica.get('changed');
unrelated.get('control');
