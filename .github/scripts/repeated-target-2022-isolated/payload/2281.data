import { primary, session } from '@vendor/valkey';
function replacement() {}
function inner(client) { client.get('key'); }
function outer(client) { inner(client); primary.get = replacement; }
outer(primary);
function innerAfter(client) { client.get('key'); }
function outerBefore(client) { session.get = replacement; innerAfter(client); }
outerBefore(session);
