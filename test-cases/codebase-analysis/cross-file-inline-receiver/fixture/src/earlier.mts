import { primary } from '@vendor/valkey';
import { factory } from './provider-earlier.mjs';
function replacement() {}
// A later supplier write must not erase this immediate cross-file read.
factory()({ client: primary });
primary.get = replacement;
