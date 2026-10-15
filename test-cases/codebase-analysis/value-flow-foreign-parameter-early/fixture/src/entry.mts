import { earlyAgreementReceiver, copiedAgreementReceiver, rhs } from '@vendor/foreign-parameter-early';
import { acquire, acquireCopied } from './helper.mts';
const outside = acquire();
outside('outside');
const copiedOutside = acquireCopied();
copiedOutside('outside');
for (const item of ['a', 'b']) {
  const inside = acquire();
  inside(item);
  const copiedInside = acquireCopied();
  copiedInside(item);
  earlyAgreementReceiver.get = rhs.get;
  copiedAgreementReceiver.get = rhs.get;
}
