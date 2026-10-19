import { primary, replica, unrelated } from '@vendor/valkey';
const box = { client: primary };
// Copying `client` drops the previous known callee. The read must not become replica.get.
Object.assign(box, { client: replica });
box.client.get('read');
primary.get('direct');
replica.get('direct');
unrelated.get('unrelated');
