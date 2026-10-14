import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
import { f } from './crossfile-provider.mjs';
f();
