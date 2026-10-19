import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
const Object = { assign() { return {}; } };
function write() { box.client.get = replacement; }
Object.assign(box, { client: replica });
// A shadowed Object.assign used as a value is not the builtin merge.
const ignored = Object.assign({}, { client: replica });
void ignored;
write();
replica.get('still-known');
