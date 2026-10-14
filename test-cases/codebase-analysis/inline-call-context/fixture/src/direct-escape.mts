import { primary } from '@vendor/valkey';
function factory() { return ({ client }) => client.get(); }
factory()({ client: primary });
// The same returned callable also has an unknown invocation through opaque code.
opaque(factory());
