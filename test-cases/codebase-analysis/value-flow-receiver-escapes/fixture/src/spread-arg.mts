import { primary, replica, untouched } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) { value.client.get = replacement; }
box = { client: replica };
// The spread stays unknown to call-target resolution; the element is still a receiver.
// An element past the last formal has no slot.
mutate(...[box, untouched]);
replica.get('changed');
primary.get('old');
untouched.get('control');
