import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function write() { box.client.get = replacement; }
// A later store cannot change the receiver already acquired by this call.
write();
box.client = replica;
replica.get('untouched');
primary.info('sibling');
