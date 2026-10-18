import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
box = { client: replica };
const captured = box;
function write() { captured.client.get = replacement; }
write();
primary.get('primary');
replica.get('replica');
