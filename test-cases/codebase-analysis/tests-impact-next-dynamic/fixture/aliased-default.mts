// `const Lazy = dynamic(...); export default Lazy;` — the lazy component is
// declared before the default-alias export. The dynamic import must still make
// `foo.mts` a transitive dependency even though the binding is visited before
// the default export marks it exported.
import dynamic from 'next/dynamic';

const Lazy = dynamic(() => import('./foo.mts'));

export default Lazy;
