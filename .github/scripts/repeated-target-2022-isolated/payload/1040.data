import { primary } from '@vendor/valkey';
declare function tick(): Promise<void>;
function replacement() {}
(async ({ client }) => { await tick(); client.get(); })({ client: primary });
primary.get = replacement;
