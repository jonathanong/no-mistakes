import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
declare const enabled: boolean;
function write() { box.client.get = replacement; }
if (enabled) box = { client: replica };
write();
primary.get('possible');
replica.get('possible');
