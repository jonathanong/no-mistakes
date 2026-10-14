import { primary, session } from '@vendor/valkey';
function replacement() {}
function inner(client) { client.get('key'); }
function outer(client = primary) { inner(client); primary.get = replacement; }
outer();
function innerAfter(client) { client.get('key'); }
function outerBefore(client = session) { session.get = replacement; innerAfter(client); }
outerBefore();
