// `next/dynamic(() => import('./foo.mts'))` — the inner dynamic import makes
// `foo.mts` a transitive dependency of this caller (a DynamicImport edge).
import dynamic from 'next/dynamic';

export const Foo = dynamic(() => import('./foo.mts'));
