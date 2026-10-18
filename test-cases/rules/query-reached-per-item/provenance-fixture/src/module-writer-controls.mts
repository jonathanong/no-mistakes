import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const copied = replica.get;
function write() { box.client.get = replacement; }
box = { client: replica };
write();
primary.get('stable');
replica.get('changed');
replica.info('sibling');
unrelated.get('unrelated');
copied('snapshot');
