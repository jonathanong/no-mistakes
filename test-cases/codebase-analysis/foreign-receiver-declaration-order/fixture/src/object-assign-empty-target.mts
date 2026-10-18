import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function write() { box.client.get = replacement; }
Object.assign({}, { client: replica });
write();
replica.get('still-known');
