import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function write() { box.client.get = replacement; }
Object.assign(box, { client: primary }, { client: replica, info: unrelated });
write();
replica.get('mutated');
unrelated.get('untouched');
