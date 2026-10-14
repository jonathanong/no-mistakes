import { primary } from '@vendor/valkey';
function replacement() {}
// Generator invocation does not execute the body until next() is called.
const iterator = (function* ({ client }) { client.get(); })({ client: primary });
primary.get = replacement;
iterator.next();
