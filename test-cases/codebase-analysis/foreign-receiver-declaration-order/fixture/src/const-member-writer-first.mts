import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
// A const binding still permits member replacement.
const box = { client: primary };
function write() { box.client.get = replacement; }
function rebind() { box.client = replica; }
rebind(); write();
replica.get('mutated');
primary.info('primary sibling');
replica.info('replica sibling');
unrelated.get('unrelated');
