import { primary } from '@vendor/valkey';
import { factory } from './provider-escaped.mjs';
function replacement() {}
const run = factory();
run({ client: primary });
primary.get = replacement;
