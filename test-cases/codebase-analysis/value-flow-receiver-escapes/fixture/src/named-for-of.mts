import { primary, replica, untouched } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) { value.client.get = replacement; }
box = { client: replica };
// Looping a named array is the same element list as `for (const item of [box])`.
const items = [box];
for (const item of items) mutate(item);
replica.get('changed');
primary.get('old');
untouched.get('control');
