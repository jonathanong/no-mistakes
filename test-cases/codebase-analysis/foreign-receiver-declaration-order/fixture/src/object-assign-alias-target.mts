import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
const target = box;
function write() { box.client.get = replacement; }
Object.assign(target, { client: replica });
write();
replica.get('mutated');
