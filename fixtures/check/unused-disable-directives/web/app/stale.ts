// no-mistakes-disable-file nextjs-no-caching: obsolete exception
// no-mistakes-disable-next-line nextjs-no-caching: no matching finding
export const value = 1;
export const other = 2; // no-mistakes-disable-line nextjs-no-caching: no finding
// no-mistakes-disable-next-line nextjs-no-api-routes: disabled rule
export const disabled = 3;
// no-mistakes-disable-next-line nextjs-no-cacheing: misspelled rule
export const typo = 4;
// no-mistakes-disable-next-line unused-disable-directive: cannot suppress hygiene
export const hygiene = 5;
