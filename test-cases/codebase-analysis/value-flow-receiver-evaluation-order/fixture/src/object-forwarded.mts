import { primary, session } from '@vendor/valkey';
function replacement() {}
function inner({ client }) { client.get('key'); }
function identifier(options) { options.client.get('key'); }
function outer(options) { inner(options); identifier(options); primary.get = replacement; }
outer({ client: primary });
function innerAfter({ client }) { client.get('key'); }
function identifierAfter(options) { options.client.get('key'); }
function outerBefore(options) { session.get = replacement; innerAfter(options); identifierAfter(options); }
outerBefore({ client: session });
