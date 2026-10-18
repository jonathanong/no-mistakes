import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function write() { box.client.get = replacement; }
Object.assign();
write();
replica.get('still-known');
