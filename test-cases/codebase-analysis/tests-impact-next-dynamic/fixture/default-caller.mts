// `export default dynamic(() => import('./foo.mts'))` — the common Next.js
// default-exported lazy component. The inner dynamic import must still make
// `foo.mts` a transitive dependency of this caller.
import dynamic from 'next/dynamic';

export default dynamic(() => import('./foo.mts'));
