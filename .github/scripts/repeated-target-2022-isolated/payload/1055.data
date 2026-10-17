import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
(({ client = primary } = {}) => client.get())();
