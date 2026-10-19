import { primary, replica, unrelated } from '@vendor/valkey';
const box = { client: primary };
// An unrelated copied field is still a member write, so the previous callee
// is Unknown. That matches `box.extra = 1; box.client.get()`.
Object.assign(box, { extra: 1 });
box.client.get('read');
primary.get('direct');
replica.get('direct');
unrelated.get('unrelated');
