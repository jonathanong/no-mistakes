// no-mistakes-disable-file nextjs-no-caching: intentionally suppress this file
export const value = fetch('/data', { cache: 'force-cache' });
