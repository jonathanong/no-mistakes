// The lazy binding is unused; the default wraps a callback that shadows its
// name. `foo.mts` must NOT be reachable from this file's test.
import dynamic from 'next/dynamic';

const Lazy = dynamic(() => import('./foo.mts'));

const wrap = (component: () => unknown) => component;

export default wrap(() => {
  const Lazy = 1;
  return Lazy;
});
