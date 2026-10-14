(({ client }) => client.get())({ client: primary });
import { primary } from '@vendor/valkey';
function replacement() {}
// The real invocation starts at byte zero, just like synthetic edge offsets.
primary.get = replacement;
