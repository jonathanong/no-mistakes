import { primary, replica, untouched } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) { value.client.get = replacement; }
box = { client: replica };
[box].map(mutate);
replica.get('changed');
primary.get('old');
untouched.get('control');
