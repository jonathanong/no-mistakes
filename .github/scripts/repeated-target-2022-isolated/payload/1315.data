import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
let unused;
holder.box.client = {};
({ first: unused = (holder.box.client.get = replacement), second: holder.other } = restore());
function restore() { holder.box = { client }; return {}; }
function globalPhaseRhsCall(value) { value.get('key'); }
globalPhaseRhsCall(client);
