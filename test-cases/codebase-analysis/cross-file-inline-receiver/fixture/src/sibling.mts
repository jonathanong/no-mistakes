import { primary } from '@vendor/valkey';
import { factory } from './provider-sibling.mjs';
// Sibling writes do not erase an earlier imported helper's get effect.
primary.ping = function replacement() {};
factory()({ client: primary });
primary.get = function replacement() {};
