import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
// A const binding still permits member replacement.
const box = { client: primary };
function rebind() { box.client = replica; }
function write() { box.client.get = replacement; }
rebind(); write();
replica.get('mutated');
primary.info('primary sibling');
replica.info('replica sibling');
unrelated.get('unrelated');
