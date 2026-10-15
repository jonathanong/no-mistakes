import { foreignOnly, forwardedReceiver, rhs } from '@vendor/source-timing';
for (const item of ['a', 'b']) {
  foreignOnly.get = rhs.get;
  forwardedReceiver.get = rhs.get;
}
