import { primary } from '@vendor/valkey';
function replacement() {}
primary.get = replacement;
(({ options }) => options.client.get())({ options: { client: primary } });
