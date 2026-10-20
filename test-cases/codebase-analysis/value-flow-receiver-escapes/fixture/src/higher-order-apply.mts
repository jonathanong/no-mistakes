import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) { value.client.get = replacement; }
function unused() {}
function apply(fn, value) { fn(value, value); }
box = { client: replica };
apply(unused, box);
apply(mutate, box);
replica.get('changed');
primary.get('old');
unrelated.get('control');
