import { sharedAlias, rhsAlias } from './barrel.mts';
import { tail } from '@vendor/producer';
sharedAlias.get = tail.get;
rhsAlias.get = tail.get;
