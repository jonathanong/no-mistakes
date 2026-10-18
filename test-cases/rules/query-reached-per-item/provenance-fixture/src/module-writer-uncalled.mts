import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function write() { box.client.get = replacement; }
box = { client: replica };
primary.get('unchanged');
replica.get('unchanged');
