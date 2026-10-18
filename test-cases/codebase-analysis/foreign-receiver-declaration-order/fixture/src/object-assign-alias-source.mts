import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
const source = { client: replica };
function write() { box.client.get = replacement; }
Object.assign(box, source);
write();
replica.get('mutated');
