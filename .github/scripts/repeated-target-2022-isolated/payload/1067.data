import { primary } from '@vendor/valkey';
declare const key: string;
function replacement() {}
primary[key] = replacement;
(({ client }) => client.get())({ client: primary });
