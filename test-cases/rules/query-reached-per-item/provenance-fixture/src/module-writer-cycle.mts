import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function write() { box.client.get = replacement; write(); }
box = { client: replica };
write();
primary.get('possible');
replica.get('possible');
