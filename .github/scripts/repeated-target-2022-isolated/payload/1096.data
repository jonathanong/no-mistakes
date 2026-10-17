import { primary } from '@vendor/valkey';
function replacement() {}
primary.get = replacement;
(({ client }) => client.get())({ client: primary });
