import { primary } from '@vendor/valkey';
import { factory } from './provider-prior.mjs';
function replacement() {}
primary.get = replacement;
factory()({ client: primary });
