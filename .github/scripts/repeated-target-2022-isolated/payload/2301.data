import { primary, session, replica, scoped } from '@vendor/valkey';
function replacement() {}
function rhs(client) { client.get('key'); return replacement; }
primary.get = rhs(primary);
session.get = replacement;
function afterWrite(client) { client.get('key'); }
afterWrite(session);
function iterable(client) { client.get('key'); return [replacement]; }
for (replica.get of iterable(replica)) {}
function keys(client) { client.get('key'); return { key: replacement }; }
for (scoped.get in keys(scoped)) {}
