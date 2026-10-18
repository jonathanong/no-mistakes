import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const captured = box;
box = { client: replica };
function write() { captured.client.get = replacement; }
write();
primary.get('primary');
replica.get('replica');
