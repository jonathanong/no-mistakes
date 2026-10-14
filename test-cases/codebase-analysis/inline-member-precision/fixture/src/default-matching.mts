import { primary } from '@vendor/valkey';
function replacement() {}
primary.get = replacement;
(({ client = primary } = {}) => client.get())();
