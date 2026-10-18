import { primary, replica } from '@vendor/valkey';
function replacement() {}
declare const condition: boolean;
const box = { client: primary };
function write() { box.client.get = replacement; }
if (condition) box.client = replica;
// A complete later store also ends a conditional provider's interval.
box.client = primary;
write();
replica.get('untouched');
primary.info('sibling');
