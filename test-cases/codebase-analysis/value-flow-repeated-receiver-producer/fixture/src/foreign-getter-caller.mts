import { foreignGetterReceiver, foreignCopiedReceiver, rhs } from '@vendor/producer';
import { acquireForeign, acquireCopied, acquireSibling } from './foreign-getter.mts';
for (const item of ['a', 'b']) {
  const fresh = acquireForeign();
  fresh(item);
  const sibling = acquireSibling();
  sibling(item, 'value');
  foreignGetterReceiver.get = rhs.get;
}
for (const item of ['a', 'b']) {
  const copied = acquireCopied();
  copied(item);
  foreignCopiedReceiver.get = rhs.get;
}
