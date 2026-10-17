import { primary } from '@vendor/valkey';
function replacement() {}
const later = (({ client }) => () => client.get())({ client: primary });
primary.get = replacement;
later();
