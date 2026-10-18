import { primary, replica, untouched } from '@vendor/forward-rebound-let';
function replacement() {}
let box = { client: primary };
let b = box;
function write() { return () => { b.client.get = replacement; }; }
b = { client: replica };
write()();
replica.get('changed');
primary.get('old');
untouched.get('control');
