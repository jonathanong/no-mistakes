import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function rebind(client) { box.client = client; }
function write() { box.client.get = replacement; }
[replica].forEach(rebind); write();
replica.get('mutated');
primary.info('primary sibling');
replica.info('replica sibling');
unrelated.get('unrelated');
