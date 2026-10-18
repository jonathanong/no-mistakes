import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const before = replica.get;
function write() { box.client.get = replacement; }
class Setup { static { box = { client: replica }; write(); } }
replica.get('changed');
before('snapshot');
