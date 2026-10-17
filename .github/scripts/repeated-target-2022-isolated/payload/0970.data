import { primary } from '@vendor/valkey';
import { factory } from './provider-other-context.mjs';
function replacement() {}
function mutate() { primary.get = replacement; }
factory()({ client: primary });
