import { primary } from '@vendor/valkey';
function factory() { return ({ client }) => client.get(); }
factory().bind(null)({ client: primary });
// Binding the immediate invocation does not remove the other opaque escape.
opaque(factory());
