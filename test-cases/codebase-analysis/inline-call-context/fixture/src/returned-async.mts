import { primary } from '@vendor/valkey';
function replacement() {}
function factory() { return async ({ client }) => { await tick(); client.get(); }; }
// The factory is synchronous; its returned helper is deferred.
factory()({ client: primary });
primary.get = replacement;
