import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function write() { box.client.get = replacement; }
box.client = replica;
// The exact store closes the earlier provider's module execution interval.
box.client = primary;
write();
replica.get('untouched');
primary.info('sibling');
