import { primary } from '@vendor/valkey';
function replacement() {}
// Invocation precedes the write, but the receiver read follows suspension.
(async ({ client }) => { await tick(); client.get(); })({ client: primary });
primary.get = replacement;
