import { primary } from '@vendor/valkey';
import { factory } from './barrel.mjs';
function replacement() {}
factory()({ client: primary });
primary.get = replacement;
