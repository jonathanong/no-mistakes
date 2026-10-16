import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
let unused;
holder.box.client = {};
({ first: unused = (holder.box.client.get = replacement), second: holder.box } = { second: { client } });
function globalPhaseBefore(value) { value.get('key'); }
globalPhaseBefore(client);
