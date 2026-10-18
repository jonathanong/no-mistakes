import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function write() { box.client.get = replacement; }
// This replaces a member, not the module root binding.
box.client = replica; write();
replica.get('mutated');
primary.info('primary sibling');
replica.info('replica sibling');
unrelated.get('unrelated');
