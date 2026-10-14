import { lateAgreementReceiver, copiedAgreementReceiver, rhs } from '@vendor/foreign-parameter-late';
import { acquire, acquireCopied } from './helper.mts';
for (const item of ['a', 'b']) {
  const inside = acquire();
  inside(item);
  const copiedInside = acquireCopied();
  copiedInside(item);
  lateAgreementReceiver.get = rhs.get;
  copiedAgreementReceiver.get = rhs.get;
}
const outside = acquire();
outside('outside');
const copiedOutside = acquireCopied();
copiedOutside('outside');
