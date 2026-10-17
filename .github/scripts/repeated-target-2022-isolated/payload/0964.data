import { primary } from '@vendor/valkey';
import { factory } from './provider-conflicting.mjs';
factory()({ client: primary });
factory()({ client: { get() {} } });
