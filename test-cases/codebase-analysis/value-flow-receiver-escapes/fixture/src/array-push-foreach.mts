import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) { value.client.get = replacement; }
box = { client: replica };
const items = [];
items.push(box);
items.forEach(mutate);
replica.get('changed');
unrelated.get('control');
