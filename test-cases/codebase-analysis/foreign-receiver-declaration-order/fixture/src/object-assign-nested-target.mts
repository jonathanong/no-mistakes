import { primary, replica } from '@vendor/valkey';
function replacement() {}
const holder = { settings: { client: primary } };
function write() { holder.settings.client.get = replacement; }
Object.assign(holder.settings, { client: replica });
write();
replica.get('mutated');
