import { primary as client } from '@vendor/valkey';
function replacement() {}
const box = { client };
if (flag) { box.client = {}; box.client.get = replacement; }
function guardedArm(value) { value.get('key'); }
guardedArm(client);
