import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
let unused;
holder.box.client = {};
({ first: unused = restore(), second: holder.box.client.get } = { second: replacement });
function restore() { holder.box = { client }; }
function globalPhaseAfterCall(value) { value.get('key'); }
globalPhaseAfterCall(client);
