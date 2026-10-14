import { primary, replica, scoped, local } from '@vendor/valkey';
function replacement() {}
function before(client) { client.get('key'); primary.get = replacement; }
before(primary);
function after(client) { replica.get = replacement; client.get('key'); }
after(replica);
const options = { client: scoped };
function objectBefore(value) { value.client.get('key'); scoped.get = replacement; }
objectBefore(options);
const later = { client: local };
function objectAfter(value) { local.get = replacement; value.client.get('key'); }
objectAfter(later);
