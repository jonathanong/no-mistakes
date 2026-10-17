import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
let unused;
holder.box.client = {};
({ first: unused = (holder.box.client.get = replacement), second: unused = restore() } = {});
function restore() { holder.box = { client }; }
function globalPhaseBeforeCall(value) { value.get('key'); }
globalPhaseBeforeCall(client);
