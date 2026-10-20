import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const source = { client: replica };
const srcs = [];
srcs.push(source);
const box = { client: primary };
function write() { box.client.get = replacement; }
Object.assign(box, ...srcs);
write();
replica.get('changed');
primary.get('old');
unrelated.get('control');
