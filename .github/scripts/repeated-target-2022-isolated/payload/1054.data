import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
(({ options = { client: primary } } = {}) => options.client.get())();
