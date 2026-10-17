import { foreignGetterReceiver, foreignCopiedReceiver, beforeLoopGetterReceiver, rhs } from '@vendor/producer';
import { acquireForeign, acquireCopied, acquireSibling, acquireBeforeLoop } from './foreign-getter.mts';
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

// The getter itself executes once here; its result remains a method snapshot.
const acquiredBefore = acquireBeforeLoop();
for (const item of ['a', 'b']) {
  acquiredBefore(item);
  beforeLoopGetterReceiver.get = rhs.get;
}
