import { facade } from './barrel.mts';
import { acquireBarrel, acquireBarrelSibling, acquireBarrelUnrelated } from './barrel-helper.mts';
import { rhs } from '@vendor/exported-value';
const copiedBarrel = acquireBarrel();
for (const item of ['a', 'b']) {
  const barrelMethod = acquireBarrel();
  barrelMethod(item);
  const barrelSibling = acquireBarrelSibling();
  barrelSibling(item);
  const barrelOther = acquireBarrelUnrelated();
  barrelOther(item);
  copiedBarrel(item);
  facade.get = rhs.get;
}
