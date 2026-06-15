// `const Lazy = dynamic(...); export default wrap(Lazy);` — the lazy binding is
// wrapped (e.g. `memo`/`forwardRef`) in the default export. The dynamic import
// must still make `foo.mts` a transitive dependency.
import dynamic from 'next/dynamic';

const wrap = (component: unknown) => component;
const Lazy = dynamic(() => import('./foo.mts'));

export default wrap(Lazy);
