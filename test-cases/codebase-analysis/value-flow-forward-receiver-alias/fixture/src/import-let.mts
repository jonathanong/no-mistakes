import { primary, replica, untouched } from '@vendor/forward-import-let';
function replacement() {}
let other = primary;
other = replica;
let box = { client: primary };
function write() { box.client.get = replacement; }
function setup() { box = { client: other }; }
write();
setup();
write();
replica.get('changed');
untouched.get('control');
