// A type and a private value share the name `Lazy`; the default only uses the
// type, so `foo.mts` must NOT be reachable from this file's test.
import dynamic from 'next/dynamic';

type Lazy = Record<string, never>;
const Lazy = dynamic(() => import('./foo.mts'));

export default {} as Lazy;
