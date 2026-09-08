import dynamic from 'next/dynamic';

// Declaring this callback does not execute it, so it must not create a
// synthetic module-root call edge. The dynamic import is still collected.
dynamic(() => import('./loaded.mts'));
