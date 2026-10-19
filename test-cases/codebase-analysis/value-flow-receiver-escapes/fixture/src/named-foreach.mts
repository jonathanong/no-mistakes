import { primary, replica, untouched } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) { value.client.get = replacement; }
box = { client: replica };
// The receiver is a named array, not a literal, so the callback still sees box.
const items = [box];
items.forEach(mutate);
// A named non-array has no element list to project.
const label = primary;
label.forEach(mutate);
replica.get('changed');
primary.get('old');
untouched.get('control');
