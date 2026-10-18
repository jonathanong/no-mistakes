import { primary, replica, untouched } from '@vendor/forward-let';
function replacement() {}
function write() { a.client.get = replacement; }
let a = { client: primary };
a = { client: replica };
write();
replica.get('changed');
primary.get('old');
untouched.get('control');
