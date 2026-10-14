import { primary, session } from '@vendor/valkey';
function replacement() {}
function factory() { return { client: primary }; }
const object = factory();
function before(options) { options.client.get('key'); }
before(object);
primary.get = replacement;
function after(options) { options.client.get('key'); }
after(object);
function storedFactory() { return { client: session }; }
const stored = storedFactory();
session.get = replacement;
function defaultAfter(options = stored) { options.client.get('key'); }
defaultAfter();
