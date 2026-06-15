// A *parenthesized* function default that shadows the outer lazy binding name in
// its body. The pre-seed must NOT treat the (unused) outer `Lazy` as exported,
// so `foo.mts` is NOT reachable from this file's test.
import dynamic from 'next/dynamic';

const Lazy = dynamic(() => import('./foo.mts'));

export default (function Page() {
  const Lazy = 1;
  return Lazy;
});
