import { primary } from '@vendor/valkey';
import { factory } from './provider.mjs';
factory()({ client: primary });
