import { primary, replica, untouched } from '@vendor/forward-var';
function replacement() {}
function write() { a.client.get = replacement; }
var a = { client: primary };
a = { client: replica };
write();
replica.get('changed');
primary.get('old');
untouched.get('control');
