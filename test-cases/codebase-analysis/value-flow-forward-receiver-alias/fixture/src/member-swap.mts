import { primary, replica, untouched } from '@vendor/forward-member-swap';
function replacement() {}
function write() { alias.client.get = replacement; }
const box = { client: primary };
const alias = box;
box.client = replica;
write();
replica.get('changed');
primary.get('old');
untouched.get('control');
