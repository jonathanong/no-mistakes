import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function write() { box.client.get = replacement; }
// Uncalled declarations remain conservative provider summaries.
function install() { Object.assign(box, { client: replica }); }
write();
replica.get('possibly-mutated');
