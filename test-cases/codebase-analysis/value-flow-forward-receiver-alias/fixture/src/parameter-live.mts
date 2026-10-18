import { primary, replica, untouched } from '@vendor/forward-parameter-live';
function replacement() {}
let box = { client: primary };
function mutate(x) { x.client.get = replacement; }
box = { client: replica };
mutate(box);
replica.get('changed');
primary.get('old');
untouched.get('control');
