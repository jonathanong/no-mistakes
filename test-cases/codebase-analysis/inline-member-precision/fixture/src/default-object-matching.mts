import { primary } from '@vendor/valkey';
function replacement() {}
primary.get = replacement;
(({ options = { client: primary } } = {}) => options.client.get())();
