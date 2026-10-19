import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: replica };
Object.assign(box.client, { get: replacement });
replica.get('mutated');
primary.get('control');
unrelated.get('unrelated');
