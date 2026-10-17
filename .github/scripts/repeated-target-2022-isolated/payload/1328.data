import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = { x: (unrelated = 1) };
holder.box.client.get = replacement;
function globalUnrelatedRhs(value) { value.get('key'); }
globalUnrelatedRhs(client);
