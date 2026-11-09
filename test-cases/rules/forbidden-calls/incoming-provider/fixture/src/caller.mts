import { primary } from '@vendor/valkey';
import { factory } from './provider.mts';
factory()({ client: primary });
