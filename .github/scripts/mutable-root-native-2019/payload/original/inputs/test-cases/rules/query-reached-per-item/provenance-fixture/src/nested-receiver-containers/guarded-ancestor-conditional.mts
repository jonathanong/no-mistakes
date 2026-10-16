import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
if (flag) { holder.box.client = { x: (other ? (holder.box = { client }) : 0) }; holder.box.client.get = replacement; }
function guardedAncestorConditional(value) { value.get('key'); }
guardedAncestorConditional(client);
