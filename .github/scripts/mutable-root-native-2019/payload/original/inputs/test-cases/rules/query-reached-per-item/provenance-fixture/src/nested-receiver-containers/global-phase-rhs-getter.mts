import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
let unused;
holder.box.client = {};
({ first: unused = (holder.box.client.get = replacement), second: holder.other } = { get first() { holder.box = { client }; return undefined; } });

function globalPhaseRhsGetter(value) { value.get('key'); }
globalPhaseRhsGetter(client);
