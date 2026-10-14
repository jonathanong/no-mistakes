import { primary } from '@vendor/valkey';
import { factory } from './provider-later.mjs';
function replacement() {}
primary.get = replacement;
factory()({ client: primary });
