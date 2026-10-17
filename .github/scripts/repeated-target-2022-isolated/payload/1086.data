import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
(({ options }) => options.client.get())({ options: { client: primary } });
