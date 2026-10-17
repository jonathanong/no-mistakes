import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
let unused;
holder.box.client = {};
({ first: holder.box, second: unused = (holder.box.client.get = replacement) } = { first: { client } });
function globalPhaseAfter(value) { value.get('key'); }
globalPhaseAfter(client);
