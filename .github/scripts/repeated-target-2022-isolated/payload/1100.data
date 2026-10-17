import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
(({ client }) => client.get())({ client: primary });
